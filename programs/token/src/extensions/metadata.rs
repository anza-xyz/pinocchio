use super::{get_extension_data_bytes_for_variable_pack, BaseState, Extension, ExtensionType};
use crate::TOKEN_2022_PROGRAM_ID;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

/// State for Metadata for a token
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct TokenMetadata<'a> {
    /// The authority that can sign to update the metadata
    pub update_authority: Pubkey,
    /// The associated mint, used to counter spoofing to be sure that metadata
    /// belongs to a particular mint
    pub mint: Pubkey,
    /// The length of the name
    pub name_len: u32,
    /// The longer name of the token
    pub name: &'a str,
    /// The length of the symbol
    pub symbol_len: u32,
    /// The shortened symbol for the token
    pub symbol: &'a str,
    /// The length of the URI
    pub uri_len: u32,
    /// The URI pointing to richer metadata
    pub uri: &'a str,
    /// The length of the additional metadata
    pub additional_metadata_len: u32,
    /// The additional metadata about the token as key-value pairs. The program
    /// must avoid storing the same key twice.
    pub additional_metadata: &'a [u8],
}

impl TokenMetadata<'_> {
    /// The fixed size of the metadata account: 80 bytes
    /// [32 (update_authority) + 32 (mint) + 4 (size of name ) + 4 (size of symbol) + 4 (size of uri) + 4 (size of additional_metadata)]
    pub const SIZE_METADATA_LEN: usize = 80;

    /// Return a `TokenMetadata` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info<'a>(
        account_info: AccountInfo,
    ) -> Result<TokenMetadata<'a>, ProgramError> {
        if account_info.data_len() < Self::SIZE_METADATA_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let account_data = account_info.try_borrow_data()?;

        let metadata_bytes =
            get_extension_data_bytes_for_variable_pack::<TokenMetadata>(account_data.as_ref())
                .ok_or(ProgramError::InvalidAccountData)?;

        Self::from_bytes(metadata_bytes)
    }

    pub(crate) fn from_bytes<'a>(data: &[u8]) -> Result<TokenMetadata<'a>, ProgramError> {
        if data.len() < Self::SIZE_METADATA_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let mut offset: usize = 0;

        let update_authority = unsafe { &*(data.as_ptr() as *const [u8; 32]) };

        offset += 32;

        let mint = unsafe { &*(data.as_ptr().add(offset) as *const [u8; 32]) };

        offset += 32;

        let name_len =
            &u32::from_le_bytes(unsafe { *(data.as_ptr().add(offset) as *const [u8; 4]) });

        offset += 4;

        let name = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                data.as_ptr().add(offset),
                *name_len as usize,
            ))
        };

        offset += *name_len as usize;

        let symbol_len =
            &u32::from_le_bytes(unsafe { *(data.as_ptr().add(offset) as *const [u8; 4]) });

        offset += 4;

        let symbol = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                data.as_ptr().add(offset),
                *symbol_len as usize,
            ))
        };

        offset += *symbol_len as usize;

        let uri_len =
            &u32::from_le_bytes(unsafe { *(data.as_ptr().add(offset) as *const [u8; 4]) });

        offset += 4;

        let uri = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                data.as_ptr().add(offset),
                *uri_len as usize,
            ))
        };

        offset += *uri_len as usize;

        let additional_metadata_len =
            &u32::from_le_bytes(unsafe { *(data.as_ptr().add(offset) as *const [u8; 4]) });

        offset += 4;

        let additional_metadata =
            unsafe { core::slice::from_raw_parts(data.as_ptr().add(offset), data.len() - offset) };

        Ok(TokenMetadata {
            update_authority: *update_authority,
            mint: *mint,
            name_len: *name_len,
            name,
            symbol_len: *symbol_len,
            symbol,
            uri_len: *uri_len,
            uri,
            additional_metadata_len: *additional_metadata_len,
            additional_metadata,
        })
    }
}

impl Extension for TokenMetadata<'_> {
    const TYPE: ExtensionType = ExtensionType::TokenMetadata;
    const LEN: usize = Self::SIZE_METADATA_LEN;
    const BASE_STATE: BaseState = BaseState::Mint;
}
