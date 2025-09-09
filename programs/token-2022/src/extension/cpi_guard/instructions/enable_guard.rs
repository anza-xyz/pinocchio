use core::{mem::MaybeUninit, slice};

use crate::{
    extension::cpi_guard::state::{
        cpi_guard_instruction_data, CpiGuardInstruction,
    },
    instructions::MAX_MULTISIG_SIGNERS,
};

use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_with_bounds,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    program_error::ProgramError,
    ProgramResult,
};

pub struct EnableCpiGuard<'a, 'b, 'c> {
    /// Token Account to update.
    pub token_account: &'a AccountInfo,
    /// Owner Account.
    pub owner: &'a AccountInfo,
    /// Signer Accounts (for multisig support)
    pub signers: &'b [&'a AccountInfo],
    /// Token Program
    pub token_program: &'c Pubkey,
}

impl EnableCpiGuard<'_, '_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let &Self {
            token_account,
            owner,
            signers,
            token_program,
        } = self;

        if signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let account_metas = [
            AccountMeta::writable_signer(token_account.key()),
            AccountMeta::readonly(owner.key()),
        ];

        let data = cpi_guard_instruction_data(CpiGuardInstruction::Enable);

        let instruction = Instruction {
            accounts: &account_metas,
            data: unsafe { slice::from_raw_parts(data.as_ptr() as _, data.len()) },
            program_id: token_program,
        };

        let num_accounts = 2 + signers.len();

        const UNINIT_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();
        let mut acc_infos = [UNINIT_INFO; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY
            acc_infos.get_unchecked_mut(0).write(token_account);
            acc_infos.get_unchecked_mut(1).write(owner);
        }

        for (account_info, signer) in acc_infos[2..].iter_mut().zip(signers.iter()) {
            account_info.write(signer);
        }

        invoke_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(&instruction, unsafe {
            slice::from_raw_parts(acc_infos.as_ptr() as _, num_accounts)
        })
    }
}
