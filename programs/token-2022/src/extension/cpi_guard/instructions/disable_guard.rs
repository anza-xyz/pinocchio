use crate::{
    extension::cpi_guard::state::{
        cpi_guard_instruction_data, CpiGuardInstruction,
    },
};
use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

pub struct DisableCpiGuard<'a, 'b> {
    /// Token Account to update.
    pub token_account: &'a AccountInfo,
    /// Owner Account.
    pub owner: &'a AccountInfo,
    /// Token Program
    pub token_program: &'b Pubkey,
}

impl DisableCpiGuard<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.token_account.key()),
            AccountMeta::readonly_signer(self.owner.key()),
        ];

        let data = cpi_guard_instruction_data(CpiGuardInstruction::Disable);

        let instruction = Instruction {
            accounts: &account_metas,
            data: &data,
            program_id: self.token_program,
        };

        invoke_signed(&instruction, &[self.token_account, self.owner], signers)
    }
}
