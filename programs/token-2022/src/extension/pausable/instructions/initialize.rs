use core::slice;

use crate::{
    extension::pausable::state::{
        pausable_initialize_instruction_data, PausableInstruction,
    },
};

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

pub struct InitializePausable<'a, 'b, 'c> {
    /// Mint Account to initialize.
    pub mint_account: &'a AccountInfo,
    /// Authority Account.
    pub authority: &'b Pubkey,
    /// Token Program
    pub token_program: &'c Pubkey,
}

impl InitializePausable<'_, '_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            mint_account,
            authority,
            token_program,
        } = self;

        let account_metas = [
            AccountMeta::writable(mint_account.key()),
        ];

        let data = pausable_initialize_instruction_data(
            PausableInstruction::Initialize,
            *authority,
        );

        let instruction = Instruction {
            accounts: &account_metas,
            data: unsafe { slice::from_raw_parts(data.as_ptr() as _, data.len()) },
            program_id: token_program,
        };

        pinocchio::cpi::invoke_signed(&instruction, &[mint_account], signers)
    }
}
