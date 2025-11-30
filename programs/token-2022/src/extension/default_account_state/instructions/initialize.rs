use core::slice;

use crate::extension::default_account_state::state::{
    default_account_state_instruction_data, DefaultAccountStateInstruction,
};

use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

pub struct InitializeDefaultAccountState<'a, 'b> {
    /// Mint Account to initialize.
    pub mint_account: &'a AccountInfo,
    /// Default state for new accounts.
    pub state: u8,
    /// Token Program
    pub token_program: &'b Pubkey,
}

impl InitializeDefaultAccountState<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            mint_account,
            state,
            token_program,
        } = self;

        let account_metas = [AccountMeta::writable(mint_account.key())];

        let data = default_account_state_instruction_data(
            DefaultAccountStateInstruction::Initialize,
            state,
        );

        let instruction = Instruction {
            accounts: &account_metas,
            data: unsafe { slice::from_raw_parts(data.as_ptr() as _, data.len()) },
            program_id: token_program,
        };

        invoke_signed(&instruction, &[mint_account], signers)
    }
}
