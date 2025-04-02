use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    program_error::ProgramError,
};

use crate::TOKEN_2022_PROGRAM_ID;

use super::get_extension_from_bytes;

/// State of the CPI guard
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CpiGuard {
    /// Lock privileged token operations from happening via CPI
    pub lock_cpi: bool,
}

impl super::Extension for CpiGuard {
    const TYPE: super::ExtensionType = super::ExtensionType::CpiGuard;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl CpiGuard {
    /// The length of the `CpiGuard` account data.
    pub const LEN: usize = core::mem::size_of::<CpiGuard>();

    /// Return a `CpiGuard` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<CpiGuard, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let acc_data_bytes = account_info.try_borrow_data()?;
        let acc_data_bytes = acc_data_bytes.as_ref();

        get_extension_from_bytes::<Self>(acc_data_bytes).ok_or(ProgramError::InvalidAccountData)
    }
}

// Instructions
pub struct EnableCpiGuard<'a> {
    /// Account to enable the CPI guard
    pub account: &'a AccountInfo,
    /// The account's owner
    pub account_owner: &'a AccountInfo,
}

impl EnableCpiGuard<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> Result<(), ProgramError> {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> Result<(), ProgramError> {
        let account_metas = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly_signer(self.account_owner.key()),
        ];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[34, 0],
        };

        invoke_signed(&instruction, &[self.account, self.account_owner], signers)?;

        Ok(())
    }
}

pub struct DisableCpiGuard<'a> {
    /// Account to disable the CPI guard
    pub account: &'a AccountInfo,
    /// The account's owner
    pub account_owner: &'a AccountInfo,
}

impl DisableCpiGuard<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> Result<(), ProgramError> {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> Result<(), ProgramError> {
        let account_metas = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly_signer(self.account_owner.key()),
        ];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[34, 1],
        };

        invoke_signed(&instruction, &[self.account, self.account_owner], signers)?;

        Ok(())
    }
}
