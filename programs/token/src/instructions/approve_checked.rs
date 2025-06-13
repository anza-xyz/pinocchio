use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

use crate::{write_bytes, InstructionData, UNINIT_BYTE};
extern crate alloc;
use alloc::boxed::Box;

/// Approves a delegate.
///
/// ### Accounts:
///   0. `[WRITE]` The source account.
///   1. `[]` The token mint.
///   2. `[]` The delegate.
///   3. `[SIGNER]` The source account owner.
pub struct ApproveChecked<'a> {
    /// Source Account.
    pub source: &'a AccountInfo,
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Delegate Account.
    pub delegate: &'a AccountInfo,
    /// Source Owner Account.
    pub authority: &'a AccountInfo,
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

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.source.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly(self.delegate.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: self.get_instruction_data(),
        };

        invoke_signed(
            &instruction,
            &[self.source, self.mint, self.delegate, self.authority],
            signers,
        )
    }
}

impl InstructionData for ApproveChecked<'_> {
    #[inline]
    fn get_instruction_data(&self) -> &[u8] {
        // Instruction data layout:
        // -  [0]  : instruction discriminator (1 byte, u8)
        // -  [1..9]: amount (8 bytes, u64)
        // -  [9]   : decimals (1 byte, u8)
        let mut instruction_data = Box::new([UNINIT_BYTE; 9]);

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data.as_mut_slice(), &[13]);
        // Set amount as u64 at offset [1..9]
        write_bytes(&mut instruction_data[1..9], &self.amount.to_le_bytes());
        // Set decimals as u8 at offset [9]
        write_bytes(&mut instruction_data[9..], &[self.decimals]);

        unsafe { from_raw_parts(instruction_data.as_ptr() as _, 10) }
    }
}
