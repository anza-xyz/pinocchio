use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    program_error::ProgramError,
    ProgramResult,
};

use crate::{state::AccountState, TOKEN_2022_PROGRAM_ID};

use super::{get_extension_from_bytes, Extension};

/// State of the default account state
#[repr(C)]
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
    /// The length of the `DefaultAccountState` account data.
    pub const LEN: usize = core::mem::size_of::<DefaultAccountState>();

    /// Return a `DefaultAccountState` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&DefaultAccountState, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(ProgramError::InvalidAccountData)
    }
}

pub struct InitializeDefaultAccountState<'a> {
    /// The mint to initialize
    pub mint: &'a AccountInfo,
    /// Default account state
    pub state: u8,
}

impl InitializeDefaultAccountState<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [2]: state (1 byte, u8)

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[28, 0, self.state],
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

impl UpdateDefaultAccountState<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.mint_freeze_authority.key()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [2]: new state (1 byte, u8)

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[28, 1, self.new_state],
        };

        invoke_signed(
            &instruction,
            &[self.mint, self.mint_freeze_authority],
            signers,
        )
    }
}
