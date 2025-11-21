use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    bytes::{write_bytes, UNINIT_BYTE},
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};

/// Mints new tokens to an account.
///
/// ### Accounts:
///   0. `[WRITE]` The mint.
///   1. `[WRITE]` The account to mint tokens to.
///   2. `[SIGNER]` The mint's minting authority.
pub struct MintToChecked<'a, 'b> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Token Account.
    pub account: &'a AccountInfo,
    /// Mint Authority
    pub mint_authority: &'a AccountInfo,
    /// Amount
    pub amount: u64,
    /// Decimals
    pub decimals: u8,
    /// Token Program
    pub token_program: &'b Pubkey,
}

impl MintToChecked<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
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
        // -  [9]: decimals (1 byte, u8)
        let mut instruction_data = [UNINIT_BYTE; 10];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[14]);
        // Set amount as u64 at offset [1..9]
        write_bytes(&mut instruction_data[1..9], &self.amount.to_le_bytes());
        // Set decimals as u8 at offset [9]
        write_bytes(&mut instruction_data[9..], &[self.decimals]);

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 10) },
        };

        invoke_signed(
            &instruction,
            &[self.mint, self.account, self.mint_authority],
            signers,
        )
    }
}
