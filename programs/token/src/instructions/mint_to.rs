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

/// Mints new tokens to an account.
///
/// ### Accounts:
///   0. `[WRITE]` The mint.
///   1. `[WRITE]` The account to mint tokens to.
///   2. `[SIGNER]` The mint's minting authority.
///
pub struct MintTo<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Token Account.
    pub account: &'a AccountInfo,
    /// Mint Authority
    pub mint_authority: &'a AccountInfo,
    /// Amount
    pub amount: u64,
}

impl MintTo<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly_signer(self.mint_authority.key()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..9]: amount (8 bytes, u64)
        let mut instruction_data = [UNINIT_BYTE; 9];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[7]);
        // Set amount as u64 at offset [1..9]
        write_bytes(&mut instruction_data[1..9], &self.amount.to_le_bytes());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 9) },
        };

        invoke_signed(
            &instruction,
            &[self.mint, self.account, self.mint_authority],
            signers,
        )
    }
}
