use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{self, AccountMeta, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

use super::get_extension_from_bytes;

/// State of the permanent delegate
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PermanentDelegate {
    /// Optional permanent delegate for transferring or burning tokens
    pub delegate: Pubkey,
}

impl super::Extension for PermanentDelegate {
    const TYPE: super::ExtensionType = super::ExtensionType::PermanentDelegate;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl PermanentDelegate {
    /// The length of the `PermanentDelegate` account data.
    pub const LEN: usize = core::mem::size_of::<PermanentDelegate>();

    /// Return a `PermanentDelegate` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&PermanentDelegate, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(ProgramError::InvalidAccountData)
    }
}

// Instructions

pub struct InitializePermanentDelegate<'a> {
    /// The mint to initialize the permanent delegate
    pub mint: &'a AccountInfo,
    /// The public key for the account that can close the mint
    pub delegate: Pubkey,
}

impl InitializePermanentDelegate<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [AccountMeta::writable(self.mint.key())];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..33]: permanent delegate (32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 33];
        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[35]);
        // Set permanent delegate as Pubkey at offset [1..33]
        write_bytes(&mut instruction_data[1..33], &self.delegate);

        let instruction = instruction::Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 33) },
        };

        invoke_signed(&instruction, &[self.mint], signers)?;

        Ok(())
    }
}
