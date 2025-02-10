use core::slice::from_raw_parts;

use {
    pinocchio::{
        account_info::AccountInfo,
        instruction::{AccountMeta, Instruction, Signer},
        ProgramResult,
    },
    pinocchio_cpi::invoke_signed,
};

use crate::{write_bytes, UNINIT_BYTE};

/// Burns tokens by removing them from an account.
///
/// ### Accounts:
///   0. `[WRITE]` The account to burn from.
///   1. `[WRITE]` The token mint.
///   2. `[SIGNER]` The account's owner/delegate.
pub struct Burn<'a> {
    /// Source of the Burn Account
    pub account: &'a AccountInfo,
    /// Mint Account
    pub mint: &'a AccountInfo,
    /// Owner of the Token Account
    pub authority: &'a AccountInfo,
    /// Amount
    pub amount: u64,
}

impl Burn<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..9]: amount (8 bytes, u64)
        let mut instruction_data = [UNINIT_BYTE; 9];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[8]);
        // Set amount as u64 at offset [1..9]
        write_bytes(&mut instruction_data[1..], &self.amount.to_le_bytes());

        let instruction = Instruction {
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
