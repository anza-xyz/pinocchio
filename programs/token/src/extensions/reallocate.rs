extern crate alloc;
use alloc::vec::Vec;
use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    ProgramResult,
};

use crate::TOKEN_2022_PROGRAM_ID;

use super::ExtensionType;

// Instruction
pub struct Reallocate<'a> {
    /// The token account to reallocate.
    pub token_account: &'a AccountInfo,
    /// The payer for the reallocation.
    pub payer: &'a AccountInfo,
    /// The system program account for reallocation.
    pub system_program: &'a AccountInfo,
    /// The token account authority.
    pub authority: &'a AccountInfo,
    /// array of extension types
    pub extension_types: &'a [ExtensionType],
}

impl Reallocate<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.token_account.key()),
            AccountMeta::writable_signer(self.payer.key()),
            AccountMeta::readonly(self.system_program.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data layout (if Field type is Key):
        // [0] : instruction discriminator
        // [1..EXTENSIONS] : extension types

        let mut instruction_data: Vec<u8> = Vec::with_capacity(1 + self.extension_types.len());

        // Write the instruction discriminator
        instruction_data.push(29);
        // Write the extension types
        for extension_type in self.extension_types {
            instruction_data.extend(extension_type.to_bytes());
        }

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(
            &instruction,
            &[
                self.token_account,
                self.payer,
                self.system_program,
                self.authority,
            ],
            signers,
        )?;

        Ok(())
    }
}
