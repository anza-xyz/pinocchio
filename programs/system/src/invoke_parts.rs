use pinocchio::{
    account_info::AccountInfo,
    cpi,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

type SliceInvokeParts<'a> = InvokeParts<&'a [&'a AccountInfo], &'a [AccountMeta<'a>], &'a [u8]>;
type FixedInvokeParts<'a, const N: usize> =
    InvokeParts<&'a [&'a AccountInfo; N], &'a [AccountMeta<'a>], &'a [u8]>;

pub trait InvokePartsType: sealed::Sealed {}
impl InvokePartsType for SliceInvokeParts<'_> {}
impl<const N: usize> InvokePartsType for FixedInvokeParts<'_, N> {}

pub struct InvokeParts<Accounts, Metas, Data> {
    pub program_id: Pubkey,
    pub accounts: Accounts,
    pub account_metas: Metas,
    pub instruction_data: Data,
}

pub trait IntoInvokeParts {
    type Output: InvokePartsType;
    fn into_invoke_parts(self) -> Self::Output;
}

pub trait Invoke: sealed::Sealed {
    fn invoke(self) -> ProgramResult;
    fn invoke_signed(self, signers: &[Signer]) -> ProgramResult;
}

impl Invoke for SliceInvokeParts<'_> {
    fn invoke(self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    fn invoke_signed(self, signers: &[Signer]) -> ProgramResult {
        cpi::slice_invoke_signed(
            &Instruction {
                program_id: &self.program_id,
                data: &self.instruction_data,
                accounts: &self.account_metas,
            },
            self.accounts,
            signers,
        )
    }
}

impl<const N: usize> Invoke for FixedInvokeParts<'_, N> {
    fn invoke(self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    fn invoke_signed(self, signers: &[Signer]) -> ProgramResult {
        cpi::invoke_signed(
            &Instruction {
                program_id: &self.program_id,
                data: &self.instruction_data,
                accounts: &self.account_metas,
            },
            self.accounts,
            signers,
        )
    }
}

impl<T> Invoke for T
where
    T: IntoInvokeParts,
    T::Output: Invoke,
{
    fn invoke(self) -> ProgramResult {
        self.into_invoke_parts().invoke()
    }

    fn invoke_signed(self, signers: &[Signer]) -> ProgramResult {
        self.into_invoke_parts().invoke_signed(signers)
    }
}

mod sealed {
    use crate::invoke_parts::{FixedInvokeParts, IntoInvokeParts, SliceInvokeParts};

    pub trait Sealed {}
    impl<'a, const N: usize> Sealed for FixedInvokeParts<'a, N> {}
    impl<'a> Sealed for SliceInvokeParts<'a> {}
    impl<T> Sealed for T where T: IntoInvokeParts {}
}
