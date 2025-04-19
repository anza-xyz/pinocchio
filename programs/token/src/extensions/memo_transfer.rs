use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    program_error::ProgramError,
};

use crate::TOKEN_2022_PROGRAM_ID;

use super::get_extension_from_bytes;

/// State of the memo transfer extension
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemoTransfer {
    /// Require transfers into this account to be accompanied by a memo
    pub require_incoming_transfer_memos: u8,
}

impl super::Extension for MemoTransfer {
    const TYPE: super::ExtensionType = super::ExtensionType::MemoTransfer;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl MemoTransfer {
    /// The length of the `MemoTranfer` account data.
    pub const LEN: usize = core::mem::size_of::<MemoTransfer>();

    /// Return a `MemoTransfer` from the given account info.
    ///
    /// This method performs owner on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&MemoTransfer, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(ProgramError::InvalidAccountData)
    }
}

// Instructions

pub struct EnableMemoTransfer<'a> {
    /// The account to update.
    pub account: &'a AccountInfo,
    /// The account owner.
    pub account_owner: &'a AccountInfo,
}

impl EnableMemoTransfer<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> Result<(), ProgramError> {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> Result<(), ProgramError> {
        // account metadata
        let account_metas = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly_signer(self.account_owner.key()),
        ];

        // Instruction data Layout
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: enable incoming transfer memos (1 byte, u8)

        let instruction = Instruction {
            program_id: &crate::TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[30, 0],
        };

        invoke_signed(&instruction, &[self.account], signers)
    }
}

pub struct DisableMemoTransfer<'a> {
    /// The account to update.
    pub account: &'a AccountInfo,
    /// The account owner.
    pub account_owner: &'a AccountInfo,
}

impl DisableMemoTransfer<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> Result<(), ProgramError> {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> Result<(), ProgramError> {
        // account metadata
        let account_metas = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly_signer(self.account_owner.key()),
        ];

        // instruction data
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: disable incoming transfer memos (1 byte, u8)

        let instruction = Instruction {
            program_id: &crate::TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[30, 1],
        };

        invoke_signed(&instruction, &[self.account, self.account_owner], signers)
    }
}
