#![no_std]
use core::usize;

use pinocchio::{
    account_info::AccountInfo,
    cpi,
    instruction::{AccountMeta, Instruction, Signer},
};

pub mod instructions;

pinocchio_pubkey::declare_id!("11111111111111111111111111111111");

pub trait InvokeParts {
    type Accounts;
    type AccountMetas;
    type InstructionData;

    fn accounts(&self) -> Self::Accounts;
    fn account_metas(&self) -> Self::AccountMetas;
    fn instruction_data(&self) -> (Self::InstructionData, usize);
}

pub trait Invoke<const N: usize>: Sized {
    fn invoke(self) -> pinocchio::ProgramResult {
        self.invoke_signed(&[])
    }

    fn invoke_signed(self, signers: &[Signer]) -> pinocchio::ProgramResult {
        self.invoke_invoker(signers, |ix, acc, sigs| cpi::invoke_signed(&ix, acc, sigs))
    }

    unsafe fn invoke_access_unchecked(self) -> pinocchio::ProgramResult {
        self.invoke_singed_access_unchecked(&[])
    }

    unsafe fn invoke_singed_access_unchecked(self, signers: &[Signer]) -> pinocchio::ProgramResult {
        self.invoke_invoker(signers, |ix, acc, sigs| unsafe {
            cpi::invoke_signed_access_unchecked(&ix, acc, sigs)
        })
    }

    fn invoke_invoker(
        self,
        signers: &[Signer],
        invoker: impl FnOnce(Instruction, &[&AccountInfo; N], &[Signer]) -> pinocchio::ProgramResult,
    ) -> pinocchio::ProgramResult;
}

impl<'a, const N: usize, const M: usize, const J: usize, T> Invoke<N> for T
where
    T: InvokeParts<
        Accounts = [&'a AccountInfo; N],
        AccountMetas = [AccountMeta<'a>; M],
        InstructionData = [u8; J],
    >,
{
    fn invoke_invoker(
        self,
        signers: &[Signer],
        invoker: impl FnOnce(Instruction, &[&AccountInfo; N], &[Signer]) -> pinocchio::ProgramResult,
    ) -> pinocchio::ProgramResult {
        let (data, end) = self.instruction_data();
        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &self.account_metas(),
            data: &data[..end],
        };
        invoker(instruction, &self.accounts(), signers)
    }
}
