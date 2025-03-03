use core::slice::from_raw_parts;

use pinocchio::{
    account_info::{AccountInfo, Ref},
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    program_error::ProgramError,
};

use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

#[derive(Debug, Clone, Copy, PartialEq)]
// State
pub struct MemoTransfer {
    /// Require transfers into this account to be accompanied by a memo
    pub require_incoming_transfer_memos: bool,
}

impl super::Extension for MemoTransfer {
    const TYPE: super::ExtensionType = super::ExtensionType::MemoTransfer;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl MemoTransfer {
    /// The length of the `MemoTranfer` account data.
    pub const LEN: usize = core::mem::size_of::<MemoTransfer>();

    /// Return a `TransferFeeConfig` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<Ref<MemoTransfer>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner() != &TOKEN_2022_PROGRAM_ID {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(Ref::map(account_info.try_borrow_data()?, |data| unsafe {
            Self::from_bytes(data)
        }))
    }

    /// Return a `MemoTransfer` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, but does not
    /// perform the borrow check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to borrow the account data – e.g., there are
    /// no mutable borrows of the account data.
    #[inline]
    pub unsafe fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&Self, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner() != &TOKEN_2022_PROGRAM_ID {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(Self::from_bytes(account_info.borrow_data_unchecked()))
    }

    /// Return a `TransferFeeConfig` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `TransferFeeConfig`.
    #[inline(always)]
    pub unsafe fn from_bytes(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const &MemoTransfer)
    }
}

// Instructions

pub struct EnableMemoTransfer<'a> {
    /// The account to update.
    pub account: &'a AccountInfo,
    /// The account owner.
    pub account_owner: &'a AccountInfo,
}

impl<'a> EnableMemoTransfer<'a> {
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
        let mut instruction_data = [UNINIT_BYTE; 2];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[30]);

        // Enable incoming transfer memos
        write_bytes(&mut instruction_data[1..2], &[0]);

        let instruction = Instruction {
            program_id: &crate::TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 2) },
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

impl<'a> DisableMemoTransfer<'a> {
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
        let mut instruction_data = [UNINIT_BYTE; 2];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[30]);
        // Disable incoming transfer memos
        write_bytes(&mut instruction_data[1..2], &[1]);

        let instruction = Instruction {
            program_id: &crate::TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 1) },
        };

        invoke_signed(&instruction, &[self.account, self.account_owner], signers)
    }
}
