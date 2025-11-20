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
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

pub struct EnableCpiGuard<'a, 'c> {
    /// Token Account to update.
    pub token_account: &'a AccountInfo,
    /// Owner Account.
    pub owner: &'a AccountInfo,
    /// Signer Accounts (for multisig support)
    pub signers: &'a [AccountInfo],
    /// Token Program
    pub token_program: &'c Pubkey,
}

impl EnableCpiGuard<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let is_multisig = !self.signers.is_empty();

        if is_multisig {
            self.invoke_multisig()
        } else {
            self.invoke_single_owner(signers)
        }
    }

    #[inline(always)]
    fn invoke_single_owner(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            token_account,
            owner,
            token_program,
            ..
        } = self;

        let account_metas = [
            AccountMeta::writable(token_account.key()),
            AccountMeta::readonly_signer(owner.key()),
        ];

        let data = cpi_guard_instruction_data(CpiGuardInstruction::Enable);

        let instruction = Instruction {
            accounts: &account_metas,
            data: unsafe { slice::from_raw_parts(data.as_ptr() as _, data.len()) },
            program_id: token_program,
        };

        pinocchio::cpi::invoke_signed(&instruction, &[token_account, owner], signers)
    }

    #[inline(always)]
    fn invoke_multisig(&self) -> ProgramResult {
        let &Self {
            token_account,
            owner,
            signers: multisig_signers,
            token_program,
        } = self;
        if multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(pinocchio::program_error::ProgramError::InvalidArgument);
        }

        let num_accounts = 2 + multisig_signers.len();

        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut acc_metas = [UNINIT_META; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY
            acc_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(token_account.key()));
            acc_metas
                .get_unchecked_mut(1)
                .write(AccountMeta::readonly(owner.key()));
        }

        for (account_meta, signer) in acc_metas[2..].iter_mut().zip(multisig_signers.iter()) {
            account_meta.write(AccountMeta::readonly_signer(signer.key()));
        }

        let data = cpi_guard_instruction_data(CpiGuardInstruction::Enable);

        let instruction = Instruction {
            accounts: unsafe { slice::from_raw_parts(acc_metas.as_ptr() as _, num_accounts) },
            data: unsafe { slice::from_raw_parts(data.as_ptr() as _, data.len()) },
            program_id: token_program,
        };

        const UNINIT_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();
        let mut acc_infos = [UNINIT_INFO; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY
            acc_infos.get_unchecked_mut(0).write(token_account);
            acc_infos.get_unchecked_mut(1).write(owner);
        }

        for (account_info, signer) in acc_infos[2..].iter_mut().zip(multisig_signers.iter()) {
            account_info.write(signer);
        }

        invoke_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(&instruction, unsafe {
            slice::from_raw_parts(acc_infos.as_ptr() as _, num_accounts)
        })
    }
}
