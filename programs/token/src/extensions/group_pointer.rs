use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

use super::{get_extension_from_bytes, BaseState, Extension, ExtensionType};

/// State of the token group pointer
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroupPointer {
    /// Authority that can set the group address
    pub authority: Pubkey,
    /// Account address that holds the group
    pub group_address: Pubkey,
}

impl GroupPointer {
    /// The length of the `GroupPointer` account data.
    pub const LEN: usize = core::mem::size_of::<GroupPointer>();

    /// Return a `GroupPointer` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<GroupPointer, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let acc_data_bytes = account_info.try_borrow_data()?;
        let acc_data_bytes = acc_data_bytes.as_ref();

        get_extension_from_bytes::<Self>(acc_data_bytes).ok_or(ProgramError::InvalidAccountData)
    }
}

impl Extension for GroupPointer {
    const TYPE: ExtensionType = ExtensionType::GroupPointer;
    const LEN: usize = Self::LEN;
    const BASE_STATE: BaseState = BaseState::Mint;
}

// TODO Initialize
// TODO Update
