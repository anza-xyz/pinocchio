use core::marker::PhantomData;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

pub type ConstAccounts<'a, const ACCOUNTS_LEN: usize> = [&'a AccountInfo; ACCOUNTS_LEN];
pub type SliceAccounts<'a> = [&'a AccountInfo];

mod sealed {
    pub trait Sealed {}

    impl<'a, const ACCOUNTS_LEN: usize> Sealed for super::ConstAccounts<'a, ACCOUNTS_LEN> {}
    impl<'a> Sealed for super::SliceAccounts<'a> {}
    impl<'a, T, Account> Sealed for super::Invoker<&'a T, Account> {}
    impl<T> Sealed for T where T: super::CanInvoke {}
}

pub trait AccountType: sealed::Sealed {}

impl<'a, const ACCOUNTS_LEN: usize> AccountType for ConstAccounts<'a, ACCOUNTS_LEN> {}
impl<'a> AccountType for SliceAccounts<'a> {}

pub trait CanInvoke {
    type Accounts: AccountType;

    fn invoke_via(
        &self,
        invoke: impl FnOnce(
            /* program_id: */ &Pubkey,
            /* accounts: */ &Self::Accounts,
            /* account_metas: */ &[AccountMeta],
            /* data: */ &[u8],
        ) -> ProgramResult,
    ) -> ProgramResult;

    #[inline]
    fn as_invoker<'a>(&'a self) -> Invoker<&'a Self, &'a Self::Accounts>
    where
        Self: Sized,
    {
        Invoker {
            inner: self,
            account_ty: PhantomData,
        }
    }
}

pub trait Invoke: sealed::Sealed {
    fn invoke(&self) -> ProgramResult;
    fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult;
}

pub struct Invoker<T, Account> {
    inner: T,
    account_ty: PhantomData<Account>,
}

impl<'a, const ACCOUNTS_LEN: usize, T> Invoke for Invoker<&'a T, &ConstAccounts<'a, ACCOUNTS_LEN>>
where
    T: CanInvoke<Accounts = ConstAccounts<'a, ACCOUNTS_LEN>>,
{
    #[inline]
    fn invoke(&self) -> ProgramResult {
        self.inner
            .invoke_via(|program_id, accounts, account_metas, data| {
                let instruction = Instruction {
                    program_id: program_id,
                    accounts: &account_metas,
                    data: data,
                };
                pinocchio::cpi::invoke(&instruction, accounts)
            })
    }

    #[inline]
    fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        self.inner
            .invoke_via(|program_id, accounts, account_metas, data| {
                let instruction = Instruction {
                    program_id: program_id,
                    accounts: &account_metas,
                    data: data,
                };
                pinocchio::cpi::invoke_signed(&instruction, accounts, signers)
            })
    }
}

impl<'a, T> Invoke for Invoker<&'a T, &SliceAccounts<'a>>
where
    T: CanInvoke<Accounts = SliceAccounts<'a>>,
{
    #[inline]
    fn invoke(&self) -> ProgramResult {
        self.inner
            .invoke_via(|program_id, accounts, account_metas, data| {
                let instruction = Instruction {
                    program_id: program_id,
                    accounts: &account_metas,
                    data: data,
                };
                pinocchio::cpi::slice_invoke(&instruction, accounts)
            })
    }

    #[inline]
    fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        self.inner
            .invoke_via(|program_id, accounts, account_metas, data| {
                let instruction = Instruction {
                    program_id: program_id,
                    accounts: &account_metas,
                    data: data,
                };
                pinocchio::cpi::slice_invoke_signed(&instruction, accounts, signers)
            })
    }
}
