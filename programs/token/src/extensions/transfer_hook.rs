use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{self, AccountMeta, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

use super::get_extension_from_bytes;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransferHookAccount {
    /// Flag to indicate that the account is in the middle of a transfer
    pub transferring: u8,
}

impl super::Extension for TransferHookAccount {
    const TYPE: super::ExtensionType = super::ExtensionType::TransferHook;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::TokenAccount;
}

impl TransferHookAccount {
    /// The length of the `TransferHookAccount` account data.
    pub const LEN: usize = core::mem::size_of::<TransferHookAccount>();

    /// Return a `TransferHookAccount` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&TransferHookAccount, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(ProgramError::InvalidAccountData)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]

pub struct TransferHook {
    /// Authority that can set the transfer hook program id
    pub authority: Pubkey,
    /// Program that authorizes the transfer
    pub program_id: Pubkey,
}

impl super::Extension for TransferHook {
    const TYPE: super::ExtensionType = super::ExtensionType::TransferHook;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl TransferHook {
    /// The length of the `TransferHook` account data.
    pub const LEN: usize = core::mem::size_of::<TransferHook>();

    /// Return a `TransferHook` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&TransferHook, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(ProgramError::InvalidAccountData)
    }
}

// Instructions
pub struct Initialize<'a> {
    /// Mint of the transfer hook
    pub mint: &'a AccountInfo,
    /// The public key for the account that can update the transfer hook program id
    pub authority: Option<Pubkey>,
    /// The program id that authorizes the transfer
    pub program_id: Option<Pubkey>,
}

impl Initialize<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> Result<(), ProgramError> {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> Result<(), ProgramError> {
        // account metas
        let account_metas = [AccountMeta::writable(self.mint.key())];

        // Instruction data layout:
        // [0] : instruction discriminator (1 byte, u8)
        // [1] : extension instruction discriminator (1 byte, u8)
        // [2..34] : authority (32 bytes, Pubkey)
        // [34..66] : program_id (32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 66];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[36]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[0]);
        // Set authority as u8 at offset [2..34]
        if let Some(authority) = self.authority {
            write_bytes(&mut instruction_data[2..34], &authority);
        } else {
            write_bytes(&mut instruction_data[2..34], &Pubkey::default());
        }
        // Set program_id as u8 at offset [34..66]
        if let Some(program_id) = self.program_id {
            write_bytes(&mut instruction_data[34..66], &program_id);
        } else {
            write_bytes(&mut instruction_data[34..66], &Pubkey::default());
        }
        let instruction = instruction::Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 66) },
        };

        invoke_signed(&instruction, &[self.mint], signers)?;

        Ok(())
    }
}

pub struct Update<'a> {
    /// Mint of the transfer hook
    pub mint: &'a AccountInfo,
    /// The public key for the account that can update the transfer hook program id
    pub authority: &'a AccountInfo,
    /// The new program id that authorizes the transfer
    pub program_id: Option<Pubkey>,
}

impl Update<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> Result<(), ProgramError> {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> Result<(), ProgramError> {
        // account metas
        let account_metas = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data layout:
        // [0] : instruction discriminator (1 byte, u8)
        // [1] : extension instruction discriminator (1 byte, u8)
        // [2..34] : authority (32 bytes, Pubkey)
        // [34..66] : program_id (32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 66];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[36]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[1]);
        // Set program_id as u8 at offset [34..66]
        if let Some(program_id) = self.program_id {
            write_bytes(&mut instruction_data[34..66], &program_id);
        } else {
            write_bytes(&mut instruction_data[34..66], &Pubkey::default());
        }
        let instruction = instruction::Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 66) },
        };

        invoke_signed(&instruction, &[self.mint], signers)?;

        Ok(())
    }
}
