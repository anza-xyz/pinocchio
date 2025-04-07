use core::slice::from_raw_parts;

use solana_account_view::AccountView;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    AccountRole, InstructionView,
};
use solana_program_error::ProgramResult;

use crate::{write_bytes, UNINIT_BYTE};

/// Burns tokens by removing them from an account.
///
/// ### Accounts:
///   0. `[WRITE]` The account to burn from.
///   1. `[WRITE]` The token mint.
///   2. `[SIGNER]` The account's owner/delegate.
pub struct Burn<'a> {
    /// Source of the Burn Account
    pub account: &'a AccountView,
    /// Mint Account
    pub mint: &'a AccountView,
    /// Owner of the Token Account
    pub authority: &'a AccountView,
    /// Amount
    pub amount: u64,
}

impl Burn<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountRole; 3] = [
            AccountRole::writable(self.account.address()),
            AccountRole::writable(self.mint.address()),
            AccountRole::readonly_signer(self.authority.address()),
        ];

        // Instruction data
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..9]: amount (8 bytes, u64)
        let mut instruction_data = [UNINIT_BYTE; 9];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[8]);
        // Set amount as u64 at offset [1..9]
        write_bytes(&mut instruction_data[1..], &self.amount.to_le_bytes());

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 9) },
        };

        invoke_signed(
            &instruction,
            &[self.account, self.mint, self.authority],
            signers,
        )
    }
}
