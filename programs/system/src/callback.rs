use core::{mem::MaybeUninit, slice};

use pinocchio::{
    account_info::AccountInfo,
    cpi::{self, MAX_CPI_ACCOUNTS},
    instruction::{Account, AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

use crate::instructions::{Transfer, TRANSFER_ACCOUNTS_LEN};

mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for T where T: super::CanInvoke {}
}

pub trait CanInvoke {
    type Accounts;

    fn invoke_via(
        &self,
        invoke: impl FnOnce(
            /* program_id: */ &Pubkey,
            /* accounts: */ &Self::Accounts,
            /* account_metas: */ &[AccountMeta],
            /* data: */ &[u8],
        ) -> ProgramResult,
        slice_invoke: impl FnOnce(
            /* program_id: */ &Pubkey,
            /* accounts: */ &[&AccountInfo],
            /* account_metas: */ &[AccountMeta],
            /* data: */ &[u8],
        ) -> ProgramResult,
    ) -> ProgramResult;
}

pub trait Invoke: sealed::Sealed {
    fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult;

    unsafe fn invoke_unchecked(&self) {
        self.invoke_signed_unchecked(&[])
    }

    unsafe fn invoke_signed_unchecked(&self, signers: &[Signer]);
}

impl<'a, const ACCOUNTS_LEN: usize, T> Invoke for T
where
    T: CanInvoke<Accounts = [&'a AccountInfo; ACCOUNTS_LEN]>,
{
    fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        self.invoke_via(
            |program_id, accounts, account_metas, data| {
                let instruction = Instruction {
                    program_id,
                    accounts: &account_metas,
                    data,
                };
                cpi::invoke_signed(&instruction, accounts, signers)
            },
            |program_id, accounts, account_metas, data| {
                let instruction = Instruction {
                    program_id,
                    accounts: &account_metas,
                    data,
                };
                cpi::slice_invoke_signed(&instruction, accounts, signers)
            },
        )
    }
    unsafe fn invoke_signed_unchecked(&self, signers: &[Signer]) {
        self.invoke_via(
            |program_id, accounts, account_metas, data| unsafe {
                let instruction = Instruction {
                    program_id,
                    accounts: &account_metas,
                    data,
                };
                cpi::invoke_signed_unchecked(&instruction, &accounts.map(Account::from), signers);
                Ok(())
            },
            |program_id, accounts, account_metas, data| unsafe {
                const UNINIT: MaybeUninit<Account> = MaybeUninit::<Account>::uninit();
                let mut ix_accounts = [UNINIT; MAX_CPI_ACCOUNTS];

                accounts.iter().enumerate().for_each(|(i, account)| {
                    ix_accounts[i] = MaybeUninit::new(Account::from(*account))
                });

                let instruction = Instruction {
                    program_id,
                    accounts: &account_metas,
                    data,
                };
                cpi::invoke_signed_unchecked(
                    &instruction,
                    slice::from_raw_parts(ix_accounts.as_ptr() as _, accounts.len()),
                    signers,
                );
                Ok(())
            },
        )
        .unwrap();
    }
}

impl<'a> CanInvoke for Transfer<'a> {
    type Accounts = [&'a AccountInfo; TRANSFER_ACCOUNTS_LEN];

    fn invoke_via(
        &self,
        invoke: impl FnOnce(
            /* program_id: */ &Pubkey,
            /* accounts: */ &Self::Accounts,
            /* account_metas: */ &[AccountMeta],
            /* data: */ &[u8],
        ) -> ProgramResult,
        _slice_invoke: impl FnOnce(
            /* program_id: */ &Pubkey,
            /* accounts: */ &[&'a AccountInfo],
            /* account_metas: */ &[AccountMeta],
            /* data: */ &[u8],
        ) -> ProgramResult,
    ) -> ProgramResult {
        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..12]: lamports amount
        let mut instruction_data = [0; 12];
        instruction_data[0] = 2;
        instruction_data[4..12].copy_from_slice(&self.lamports.to_le_bytes());

        invoke(
            &crate::ID,
            &[self.from, self.to],
            &[
                AccountMeta::writable_signer(self.from.key()),
                AccountMeta::writable(self.to.key()),
            ],
            &instruction_data,
        )
    }
}
