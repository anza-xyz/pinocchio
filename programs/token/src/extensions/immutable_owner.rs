use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    ProgramResult,
};

use crate::TOKEN_2022_PROGRAM_ID;

use super::get_extension_from_bytes;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImmutableOwner;

impl super::Extension for ImmutableOwner {
    const TYPE: super::ExtensionType = super::ExtensionType::ImmutableOwner;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::TokenAccount;
}

impl ImmutableOwner {
    /// The length of the `ImmutableOwner` account data.
    pub const LEN: usize = core::mem::size_of::<ImmutableOwner>();

    /// Return a `ImmutableOwner` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<ImmutableOwner, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let acc_data_bytes = account_info.try_borrow_data()?;
        let acc_data_bytes = acc_data_bytes.as_ref();

        get_extension_from_bytes::<Self>(acc_data_bytes).ok_or(ProgramError::InvalidAccountData)
    }
}

// Instructions
pub struct InitializeImmutableOwner<'a> {
    /// The mint to initialize the non-transferable
    pub mint: &'a AccountInfo,
}

impl InitializeImmutableOwner<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [AccountMeta::writable(self.mint.key())];

        // Instruction data Layout:
        // -  [0]: instruction discriminator
        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[22],
        };

        invoke_signed(&instruction, &[self.mint], signers)?;

        Ok(())
    }
}
