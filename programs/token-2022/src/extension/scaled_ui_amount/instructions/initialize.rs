use core::slice;

use crate::{
    extension::scaled_ui_amount::state::{
        scaled_ui_amount_instruction_data, ScaledUiAmountInstruction,
    },
};

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

pub struct InitializeScaledUiAmount<'a, 'b> {
    /// Mint Account to initialize.
    pub mint_account: &'a AccountInfo,
    /// Token Program
    pub token_program: &'b Pubkey,
}

impl InitializeScaledUiAmount<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            mint_account,
            token_program,
        } = self;

        let account_metas = [
            AccountMeta::writable(mint_account.key()),
        ];

        let data = scaled_ui_amount_instruction_data(ScaledUiAmountInstruction::Initialize);

        let instruction = Instruction {
            accounts: &account_metas,
            data: unsafe { slice::from_raw_parts(data.as_ptr() as _, data.len()) },
            program_id: token_program,
        };

        pinocchio::cpi::invoke_signed(&instruction, &[mint_account], signers)
    }
}
