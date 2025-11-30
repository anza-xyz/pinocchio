use crate::{
    extension::transfer_hook::state::{
        transfer_hook_initialize_instruction_data, TransferHookInstruction,
    },
};

use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

pub struct InitializeTransferHook<'a> {
    /// Mint Account to initialize.
    pub mint_account: &'a AccountInfo,
    /// Optional authority that can set the transfer hook program id
    pub authority: Option<&'a Pubkey>,
    /// Program that authorizes the transfer
    pub program_id: Option<&'a Pubkey>,
    /// Token Program
    pub token_program: &'a Pubkey,
}

impl InitializeTransferHook<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [AccountMeta::writable(self.mint_account.key())];

        let mut buffer = [0u8; 66];
        let data = transfer_hook_initialize_instruction_data(
            &mut buffer,
            TransferHookInstruction::Initialize,
            self.authority,
            self.program_id,
        );

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data,
        };

        invoke_signed(&instruction, &[self.mint_account], signers)
    }
}
