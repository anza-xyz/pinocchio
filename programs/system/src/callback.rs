use pinocchio::{
    account_info::AccountInfo,
    cpi,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for T where T: super::CanInvoke {}
}

pub trait CanInvoke {
    type Accounts;

    fn invoke_via(
        &self,
        invoke: impl for<'a> FnOnce(
            /* program_id: */ &'a Pubkey,
            /* accounts: */ &'a Self::Accounts,
            /* account_metas: */ &'a [AccountMeta],
            /* data: */ &'a [u8],
        ) -> ProgramResult,
        slice_invoke: impl for<'a> FnOnce(
            /* program_id: */ &'a Pubkey,
            /* accounts: */ &'a [&'a AccountInfo],
            /* account_metas: */ &'a [AccountMeta],
            /* data: */ &'a [u8],
        ) -> ProgramResult,
    ) -> ProgramResult;
}

pub trait Invoke: sealed::Sealed {
    fn invoke(&self) -> ProgramResult;
    fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult;
}

impl<'a, const ACCOUNTS_LEN: usize, T> Invoke for T
where
    T: CanInvoke<Accounts = [&'a AccountInfo; ACCOUNTS_LEN]>,
{
    fn invoke(&self) -> ProgramResult {
        self.invoke_via(
            |program_id, accounts, account_metas, data| {
                let instruction = Instruction {
                    program_id,
                    accounts: &account_metas,
                    data,
                };
                cpi::invoke(&instruction, accounts)
            },
            |program_id, accounts, account_metas, data| {
                let instruction = Instruction {
                    program_id,
                    accounts: &account_metas,
                    data,
                };
                cpi::slice_invoke(&instruction, accounts)
            },
        )
    }

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
}
