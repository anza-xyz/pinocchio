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

/// Transfer Tokens from one Token Account to another.
///
/// ### Accounts:
///   0. `[WRITE]` Sender account
///   1. `[WRITE]` Recipient account
///   2. `[SIGNER]` Authority account
pub struct Transfer<'a> {
    /// Sender account.
    pub from: &'a AccountInfo,
    /// Recipient account.
    pub to: &'a AccountInfo,
    /// Authority account.
    pub authority: &'a AccountInfo,
    /// Amount of microtokens to transfer.
    pub amount: u64,
}

impl Transfer<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.from.key()),
            AccountMeta::writable(self.to.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: self.get_instruction_data(),
        };

        invoke_signed(&instruction, &[self.from, self.to, self.authority], signers)
    }
}

impl InstructionData for Transfer<'_> {
    #[inline]
    fn get_instruction_data(&self) -> &[u8] {
        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..9]: amount (8 bytes, u64)
        let mut instruction_data = Box::new([UNINIT_BYTE; 9]);
        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data.as_mut_slice(), &[3]);
        // Set amount as u64 at offset [1..9]
        write_bytes(&mut instruction_data[1..9], &self.amount.to_le_bytes());
        unsafe { from_raw_parts(instruction_data.as_ptr() as _, 9) }
    }
}
