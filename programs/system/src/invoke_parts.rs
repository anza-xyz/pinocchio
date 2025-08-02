use pinocchio::{
    account_info::AccountInfo,
    cpi,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

use crate::instructions::{Transfer, TRANSFER_ACCOUNTS_LEN, TRANSFER_DATA_SIZE};

type SliceInvokeParts<'a> = InvokeParts<&'a [&'a AccountInfo], &'a [AccountMeta<'a>], &'a [u8]>;
type FixedInvokeParts<'a, const N: usize, const M: usize> =
    InvokeParts<[&'a AccountInfo; N], [AccountMeta<'a>; N], [u8; M]>;

pub trait InvokePartsType: sealed::Sealed {}
impl InvokePartsType for SliceInvokeParts<'_> {}
impl<const N: usize, const M: usize> InvokePartsType for FixedInvokeParts<'_, N, M> {}

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

impl<const N: usize, const M: usize> Invoke for FixedInvokeParts<'_, N, M> {
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
            &self.accounts,
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
    impl<'a, const N: usize, const M: usize> Sealed for FixedInvokeParts<'a, N, M> {}
    impl<'a> Sealed for SliceInvokeParts<'a> {}
    impl<T> Sealed for T where T: IntoInvokeParts {}
}

impl<'a> IntoInvokeParts for Transfer<'a> {
    type Output = FixedInvokeParts<'a, TRANSFER_ACCOUNTS_LEN, TRANSFER_DATA_SIZE>;

    fn into_invoke_parts(self) -> Self::Output {
        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..12]: lamports amount
        let mut instruction_data = [0; 12];
        instruction_data[0] = 2;
        instruction_data[4..12].copy_from_slice(&self.lamports.to_le_bytes());
        FixedInvokeParts {
            program_id: crate::ID,
            accounts: [&self.from, &self.to],
            account_metas: [
                AccountMeta::writable_signer(self.from.key()),
                AccountMeta::writable(self.to.key()),
            ],
            instruction_data: instruction_data,
        }
    }
}
