use {
    super::constants::MAX_IX_DATA,
    crate::{write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Remove a key-value pair from token metadata.
///
/// This instruction removes a custom key from the additional metadata.
/// If idempotent is true, the instruction succeeds even if the key doesn't
/// exist.
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
        let ix_len = 8 + 1 + 4 + self.key.len();

        if ix_len > MAX_IX_DATA {
            return Err(ProgramError::InvalidArgument);
        }

        let mut ix_data = [UNINIT_BYTE; MAX_IX_DATA];
        let mut offset = 0;

        // Set 8-byte discriminator for RemoveKey
        write_bytes(&mut ix_data[offset..offset + 8], &Self::DISCRIMINATOR);
        offset += 8;

        // Set idempotent flag
        write_bytes(&mut ix_data[offset..offset + 1], &[self.idempotent as u8]);
        offset += 1;

        // Set serialized key data
        write_bytes(
            &mut ix_data[offset..offset + 4],
            &(self.key.len() as u32).to_le_bytes(),
        );
        offset += 4;
        write_bytes(
            &mut ix_data[offset..offset + self.key.len()],
            self.key.as_bytes(),
        );

        let instruction_accounts: [InstructionAccount; 2] = [
            InstructionAccount::writable(self.metadata.address()),
            InstructionAccount::readonly_signer(self.update_authority.address()),
        ];

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &instruction_accounts,
            data: unsafe { from_raw_parts(ix_data.as_ptr() as _, ix_len) },
        };

        invoke_signed(
            &instruction,
            &[self.metadata, self.update_authority],
            signers,
        )
    }
}
