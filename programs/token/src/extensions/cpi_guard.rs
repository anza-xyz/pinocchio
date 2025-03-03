use core::slice::from_raw_parts;

use pinocchio::{
    account_info::{AccountInfo, Ref},
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    program_error::ProgramError,
};

use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

pub struct CpiGuard {
    /// Lock privileged token operations from happening via CPI
    pub lock_cpi: bool,
}

impl CpiGuard {
    /// The length of the `CpiGuard` account data.
    pub const LEN: usize = core::mem::size_of::<CpiGuard>();

    /// Return a `CpiGuard` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<Ref<CpiGuard>, ProgramError> {
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

    /// Return a `CpiGuard` from the given account info.
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

    /// Return a `CpiGuard` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `CpiGuard`.
    #[inline(always)]
    pub unsafe fn from_bytes(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const &CpiGuard)
    }
}

// Instructions
pub struct EnableCpiGuard<'a> {
    /// Account to enable the CPI guard
    pub account: &'a AccountInfo,
    /// The account's owner
    pub account_owner: &'a AccountInfo,
}

impl<'a> EnableCpiGuard<'a> {
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
        let mut instruction_data = [UNINIT_BYTE; 2];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[34]);

        // Enable the CPI guard
        write_bytes(&mut instruction_data[1..2], &[0]);

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 2) },
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

impl<'a> DisableCpiGuard<'a> {
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
        let mut instruction_data = [UNINIT_BYTE; 2];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[34]);

        // Disable the CPI guard
        write_bytes(&mut instruction_data[1..2], &[1]);

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 2) },
        };

        invoke_signed(&instruction, &[self.account, self.account_owner], signers)?;

        Ok(())
    }
}
