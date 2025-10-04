use core::slice::from_raw_parts;

use solana_account_view::AccountView;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    AccountMeta, InstructionView,
};
use solana_program_error::ProgramResult;

use crate::{write_bytes, UNINIT_BYTE};

/// Approves a delegate.
///
/// ### Accounts:
///   0. `[WRITE]` The source account.
///   1. `[]` The token mint.
///   2. `[]` The delegate.
///   3. `[SIGNER]` The source account owner.
pub struct ApproveChecked<'a> {
    /// Source Account.
    pub source: &'a AccountView,
    /// Mint Account.
    pub mint: &'a AccountView,
    /// Delegate Account.
    pub delegate: &'a AccountView,
    /// Source Owner Account.
    pub authority: &'a AccountView,
    /// Amount.
    pub amount: u64,
    /// Decimals.
    pub decimals: u8,
}

impl ApproveChecked<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.source.address()),
            AccountMeta::readonly(self.mint.address()),
            AccountMeta::readonly(self.delegate.address()),
            AccountMeta::readonly_signer(self.authority.address()),
        ];

        // Instruction data
        // -  [0]  : instruction discriminator (1 byte, u8)
        // -  [1..9]: amount (8 bytes, u64)
        // -  [9]   : decimals (1 byte, u8)
        let mut instruction_data = [UNINIT_BYTE; 10];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[13]);
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
            &[self.source, self.mint, self.delegate, self.authority],
            signers,
        )
    }
}
