use core::slice::from_raw_parts;

use solana_account_view::AccountView;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    AccountMeta, InstructionView,
};
use solana_program_error::ProgramResult;

use crate::{write_bytes, UNINIT_BYTE};

/// Transfer Tokens from one Token Account to another.
///
/// ### Accounts:
///   0. `[WRITE]` The source account.
///   1. `[]` The token mint.
///   2. `[WRITE]` The destination account.
///   3. `[SIGNER]` The source account's owner/delegate.
pub struct TransferChecked<'a> {
    /// Sender account.
    pub from: &'a AccountView,
    /// Mint Account
    pub mint: &'a AccountView,
    /// Recipient account.
    pub to: &'a AccountView,
    /// Authority account.
    pub authority: &'a AccountView,
    /// Amount of micro-tokens to transfer.
    pub amount: u64,
    /// Decimal for the Token
    pub decimals: u8,
}

impl TransferChecked<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.from.address()),
            AccountMeta::readonly(self.mint.address()),
            AccountMeta::writable(self.to.address()),
            AccountMeta::readonly_signer(self.authority.address()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..9]: amount (8 bytes, u64)
        // -  [9]: decimals (1 byte, u8)
        let mut instruction_data = [UNINIT_BYTE; 10];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[12]);
        // Set amount as u64 at offset [1..9]
        write_bytes(&mut instruction_data[1..9], &self.amount.to_le_bytes());
        // Set decimals as u8 at offset [9]
        write_bytes(&mut instruction_data[9..], &[self.decimals]);

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 10) },
        };

        invoke_signed(
            &instruction,
            &[self.from, self.mint, self.to, self.authority],
            signers,
        )
    }
}
