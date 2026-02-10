// NOTE: Metadata interface instructions use `Vec` for instruction data because
// the payload contains variable-length strings whose total size is not known at
// compile time.  The rest of the crate uses stack-allocated `UNINIT_BYTE` arrays,
// which is possible only when the maximum data size is bounded and small.
extern crate alloc;

use alloc::vec::Vec;
use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    InstructionAccount, InstructionView,
};
use solana_program_error::ProgramResult;

/// Remove a key-value pair from token metadata.
///
/// This instruction removes a custom key from the additional metadata.
/// If idempotent is true, the instruction succeeds even if the key doesn't exist.
///
/// ### Accounts:
///   0. `[WRITE]` Metadata account
///   1. `[SIGNER]` Update authority
pub struct RemoveKey<'a, 'b> {
    /// The metadata account to update
    pub metadata: &'a AccountView,
    /// The account authorized to update the metadata
    pub update_authority: &'a AccountView,
    /// The key to remove from the metadata
    pub key: &'a str,
    /// Whether the operation should be idempotent
    pub idempotent: bool,
    /// Token program (Token-2022).
    pub token_program: &'b Address,
}

impl RemoveKey<'_, '_> {
    /// Based on `spl_token_metadata_interface` hash.
    pub const DISCRIMINATOR: [u8; 8] = [234, 18, 32, 56, 89, 141, 37, 181];

    /// Invoke the `RemoveKey` instruction.
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    /// Invoke the `RemoveKey` instruction with signers.
    ///
    /// Instruction data layout:
    /// - `[0..8]`: instruction discriminator (8 bytes)
    /// - `[8..9]`: idempotent flag (1 byte, bool as `u8`)
    /// - `[9..13]`: key length (4 bytes, `u32`)
    /// - `[13..13+K]`: key string (K bytes, UTF-8)
    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let ix_len = 8 // instruction discriminator
            + 1 // idempotent flag
            + 4 // key length
            + self.key.len(); // key data

        let mut ix_data: Vec<u8> = Vec::with_capacity(ix_len);

        // Set 8-byte discriminator for RemoveKey
        ix_data.extend_from_slice(&Self::DISCRIMINATOR);

        // Set idempotent flag
        ix_data.push(self.idempotent as u8);

        // Set serialized key data
        let key_len = self.key.len() as u32;
        ix_data.extend_from_slice(&key_len.to_le_bytes());
        ix_data.extend_from_slice(self.key.as_bytes());

        // Create instruction accounts
        let instruction_accounts: [InstructionAccount; 2] = [
            InstructionAccount::writable(self.metadata.address()),
            InstructionAccount::readonly_signer(self.update_authority.address()),
        ];

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &instruction_accounts,
            data: &ix_data,
        };

        invoke_signed(
            &instruction,
            &[self.metadata, self.update_authority],
            signers,
        )
    }
}
