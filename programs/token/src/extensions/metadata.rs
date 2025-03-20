use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::TOKEN_2022_PROGRAM_ID;

use super::{get_extension_from_bytes, BaseState, Extension, ExtensionType};

#[derive(Debug, Clone, Copy, PartialEq)]
/// Metadata for a token
pub struct TokenMetadata<'s> {
    /// The authority that can sign to update the metadata
    pub update_authority: Pubkey,
    /// The associated mint, used to counter spoofing to be sure that metadata
    /// belongs to a particular mint
    pub mint: Pubkey,
    /// The longer name of the token
    pub name: &'s str,
    /// The shortened symbol for the token
    pub symbol: &'s str,
    /// The URI pointing to richer metadata
    pub uri: &'s str,
    /// Any additional metadata about the token as key-value pairs. The program
    /// must avoid storing the same key twice.
    pub additional_metadata: &'s [(&'s str, &'s str)],
}

impl<'t> TokenMetadata<'t> {
    /// The length of the `TokenMetadata` account data.
    pub const LEN: usize = core::mem::size_of::<TokenMetadata>();

    /// Return a `TokenMetadata` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(
        account_info: &'t AccountInfo,
    ) -> Result<TokenMetadata<'t>, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let acc_data_bytes = account_info.try_borrow_data()?;
        let acc_data_bytes = acc_data_bytes.as_ref();

        get_extension_from_bytes::<Self>(acc_data_bytes).ok_or(ProgramError::InvalidAccountData)
    }
}

impl Extension for TokenMetadata<'_> {
    const TYPE: ExtensionType = ExtensionType::TokenMetadata;
    const LEN: usize = Self::LEN;
    const BASE_STATE: BaseState = BaseState::Mint;
}
