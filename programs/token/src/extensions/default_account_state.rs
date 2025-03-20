use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    program_error::ProgramError,
    ProgramResult,
};

use crate::{state::AccountState, write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

use super::{get_extension_from_bytes, Extension};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DefaultAccountState {
    pub state: AccountState,
}

impl Extension for DefaultAccountState {
    const TYPE: super::ExtensionType = super::ExtensionType::DefaultAccountState;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl DefaultAccountState {
    /// The length of the `MemoTranfer` account data.
    pub const LEN: usize = core::mem::size_of::<DefaultAccountState>();

    /// Return a `DefaultAccountState` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<DefaultAccountState, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let acc_data_bytes = account_info.try_borrow_data()?;
        let acc_data_bytes = acc_data_bytes.as_ref();

        get_extension_from_bytes::<Self>(acc_data_bytes).ok_or(ProgramError::InvalidAccountData)
    }
}

pub struct InitializeDefaultAccountState<'a> {
    /// The mint to initialize
    pub mint: &'a AccountInfo,
    /// Default account state
    pub state: u8,
}

impl<'a> InitializeDefaultAccountState<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        let mut instruction_data = [UNINIT_BYTE; 1];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[28]);

        write_bytes(&mut instruction_data[1..2], &[self.state]);

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 1) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}

pub struct UpdateDefaultAccountState<'a> {
    /// The mint to update
    pub mint: &'a AccountInfo,
    /// The mint's freeze authority
    pub mint_freeze_authority: &'a AccountInfo,
    /// The new state
    pub new_state: u8,
}
