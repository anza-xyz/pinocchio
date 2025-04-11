use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

use super::get_extension_from_bytes;

/// State of the pausable mint
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PausableConfig {
    /// Authority that can pause or resume activity on the mint
    pub authority: Pubkey,
    /// Whether minting / transferring / burning tokens is paused
    pub paused: bool,
}

impl super::Extension for PausableConfig {
    const TYPE: super::ExtensionType = super::ExtensionType::Pausable;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl PausableConfig {
    /// The length of the `PausableConfig` account data.
    pub const LEN: usize = core::mem::size_of::<PausableConfig>();

    /// Return a `PausableConfig` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&PausableConfig, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(ProgramError::InvalidAccountData)
    }
}

/// State of the pausable token account
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PausableAccount;

impl super::Extension for PausableAccount {
    const TYPE: super::ExtensionType = super::ExtensionType::PausableAccount;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::TokenAccount;
}

impl PausableAccount {
    /// The length of the `PausableAccount` account data.
    pub const LEN: usize = core::mem::size_of::<PausableAccount>();

    /// Return a `PausableAccount` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&PausableAccount, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(ProgramError::InvalidAccountData)
    }
}

// Instructions

pub struct InitializePausable<'a> {
    /// The mint to initialize the pausable config
    pub mint: &'a AccountInfo,
    /// The public key for the account that can pause or resume activity on the mint
    pub authority: Pubkey,
}

impl InitializePausable<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [AccountMeta::writable(self.mint.key())];

        // Instruction data Layout:
        //[0] u8: instruction discriminator
        //[1] u8: extension instruction discriminator
        //[2..34] u8: authority

        let mut instruction_data = [UNINIT_BYTE; 34];

        // Set the instruction discriminator
        write_bytes(&mut instruction_data[0..1], &[44]);
        // Set the extension ix discriminator
        write_bytes(&mut instruction_data[1..2], &[0]);
        // Set the authority
        write_bytes(&mut instruction_data[2..34], &self.authority);

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 34) },
        };

        invoke_signed(&instruction, &[self.mint], signers)?;

        Ok(())
    }
}

pub struct Pause<'a> {
    /// The mint to pause
    pub mint: &'a AccountInfo,
    // The mint's pause authority
    pub pause_authority: &'a AccountInfo,
}

impl Pause<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [AccountMeta::writable(self.mint.key())];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[45, 1],
        };

        invoke_signed(&instruction, &[self.mint, self.pause_authority], signers)?;

        Ok(())
    }
}

pub struct Resume<'a> {
    /// The mint to unpause
    pub mint: &'a AccountInfo,
    // The mint's pause authority
    pub pause_authority: &'a AccountInfo,
}

impl Resume<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [AccountMeta::writable(self.mint.key())];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[45, 2],
        };

        invoke_signed(&instruction, &[self.mint, self.pause_authority], signers)?;

        Ok(())
    }
}
