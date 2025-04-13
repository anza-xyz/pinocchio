use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, InstructionData, UNINIT_BYTE};
extern crate alloc;
use alloc::boxed::Box;

/// Initialize a new Token Account.
///
/// ### Accounts:
///   0. `[WRITE]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
pub struct InitializeAccount3<'a> {
    /// New Account.
    pub account: &'a AccountInfo,
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Owner of the new Account.
    pub owner: &'a Pubkey,
}

impl InitializeAccount3<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly(self.mint.key()),
        ];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: self.get_instruction_data(),
        };

        invoke_signed(&instruction, &[self.account, self.mint], signers)
    }
}

impl InstructionData for InitializeAccount3<'_> {
    #[inline]
    fn get_instruction_data(&self) -> &[u8] {
        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..33]: owner (32 bytes, Pubkey)
        let mut instruction_data = Box::new([UNINIT_BYTE; 33]);

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data.as_mut_slice(), &[18]);
        // Set owner as [u8; 32] at offset [1..33]
        write_bytes(&mut instruction_data[1..], self.owner);

        unsafe { from_raw_parts(instruction_data.as_ptr() as _, 33) }
    }
}
