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

/// Approves a delegate.
///
/// ### Accounts:
///   0. `[WRITE]` The token account.
///   1. `[]` The delegate.
///   2. `[SIGNER]` The source account owner.
pub struct Approve<'a> {
    /// Source Account.
    pub source: &'a AccountInfo,
    /// Delegate Account
    pub delegate: &'a AccountInfo,
    /// Source Owner Account
    pub authority: &'a AccountInfo,
    /// Amount
    pub amount: u64,
}

impl Approve<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.source.key()),
            AccountMeta::readonly(self.delegate.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..9]: amount (8 bytes, u64)
        let mut instruction_data = [UNINIT_BYTE; 9];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[4]);
        // Set amount as u64 at offset [1..9]
        write_bytes(&mut instruction_data[1..], &self.amount.to_le_bytes());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 9) },
        };

        invoke_signed(
            &instruction,
            &[self.source, self.delegate, self.authority],
            signers,
        )
    }
}
