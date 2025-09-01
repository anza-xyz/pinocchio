use crate::{
    extensions::default_account_state::state::{
        encode_instruction_data, DefaultAccountStateInstruction,
    },
    state::AccountState,
};
use core::slice::from_raw_parts;
use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

pub struct UpdateDefaultAccountState<'a, 'b> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Freeze Authority Account.
    pub freeze_authority: &'a AccountInfo,
    /// Token Program
    pub token_program: &'b Pubkey,
    /// Account State
    pub state: AccountState,
}

impl UpdateDefaultAccountState<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.freeze_authority.key()),
        ];

        let data = encode_instruction_data(DefaultAccountStateInstruction::Update, self.state);

        let instruction = Instruction {
            accounts: &account_metas,
            data: unsafe { from_raw_parts(data.as_ptr() as _, data.len()) },
            program_id: self.token_program,
        };

        invoke_signed(&instruction, &[self.mint, self.freeze_authority], signers)
    }
}
