use super::{get_extension_data_bytes_for_variable_pack, BaseState, Extension, ExtensionType};
use crate::std::{
    convert::TryInto,
    string::{String, ToString},
    vec::Vec,
};
use crate::TOKEN_2022_PROGRAM_ID;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

/// State for Metadata for a token
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct TokenMetadata {
    /// The authority that can sign to update the metadata
    pub update_authority: Pubkey,
    /// The associated mint, used to counter spoofing to be sure that metadata
    /// belongs to a particular mint
    pub mint: Pubkey,
    /// The longer name of the token
    pub name: String,
    /// The shortened symbol for the token
    pub symbol: String,
    /// The URI pointing to richer metadata
    pub uri: String,
    /// Any additional metadata about the token as key-value pairs. The program
    /// must avoid storing the same key twice.
    pub additional_metadata: Vec<(String, String)>,
}

impl TokenMetadata {
    /// The length of the `TokenMetadata` account data.
    pub const LEN: usize = core::mem::size_of::<TokenMetadata>();

    /// Return a `TokenMetadata` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(account_info: AccountInfo) -> Result<TokenMetadata, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let acc_data_bytes = account_info.try_borrow_data()?;
        let acc_data_bytes = acc_data_bytes.as_ref();

        let ext_bytes = get_extension_data_bytes_for_variable_pack::<TokenMetadata>(acc_data_bytes)
            .ok_or(ProgramError::InvalidAccountData)?;

        Self::unpack_from_ext_bytes(ext_bytes)
    }

    pub(crate) fn unpack_from_ext_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        // 80 = 32 (update_authority) + 32 (mint) + 4 (size of name ) + 4 (size of symbol) + 4 (size of uri) + 4 (size of additional_metadata)
        if data.len() < 80 {
            return Err(ProgramError::InvalidAccountData);
        }

        let mut current_offset = 0usize;

        let update_authority: Pubkey = data[0..32].try_into().unwrap();

        current_offset += 32;

        let mint: Pubkey = data[current_offset..current_offset + 32]
            .try_into()
            .unwrap();

        current_offset += 32;

        let name_len = u32::from_le_bytes(
            data[current_offset..(current_offset + 4)]
                .try_into()
                .unwrap(),
        ) as usize;
        current_offset += 4;

        let name = core::str::from_utf8(&data[current_offset..(current_offset + name_len)])
            .unwrap()
            .to_string();

        current_offset += name_len;

        let symbol_len = u32::from_le_bytes(
            data[current_offset..(current_offset + 4)]
                .try_into()
                .unwrap(),
        ) as usize;

        current_offset += 4;

        let symbol = core::str::from_utf8(&data[current_offset..(current_offset + symbol_len)])
            .unwrap()
            .to_string();

        current_offset += symbol_len;

        let uri_len = u32::from_le_bytes(
            data[current_offset..(current_offset + 4)]
                .try_into()
                .unwrap(),
        ) as usize;

        current_offset += 4;

        let uri = core::str::from_utf8(&data[current_offset..(current_offset + uri_len)])
            .unwrap()
            .to_string();

        current_offset += uri_len;

        let additional_metadata_len = u32::from_le_bytes(
            data[current_offset..(current_offset + 4)]
                .try_into()
                .unwrap(),
        ) as usize;

        current_offset += 4;

        let additional_metadata_bytes =
            &data[current_offset..(current_offset + additional_metadata_len)];

        let mut additional_metadata: Vec<(String, String)> =
            Vec::with_capacity(additional_metadata_len);

        for _ in 0..additional_metadata_len {
            let key_len = u32::from_le_bytes(
                additional_metadata_bytes[current_offset..(current_offset + 4)]
                    .try_into()
                    .unwrap(),
            ) as usize;

            current_offset += 4;

            let key = core::str::from_utf8(
                &additional_metadata_bytes[current_offset..(current_offset + key_len)],
            )
            .unwrap()
            .to_string();

            current_offset += key_len;

            let value_len = u32::from_le_bytes(
                additional_metadata_bytes[current_offset..(current_offset + 4)]
                    .try_into()
                    .unwrap(),
            ) as usize;

            current_offset += 4;

            let value = core::str::from_utf8(
                &additional_metadata_bytes[current_offset..(current_offset + value_len)],
            )
            .unwrap()
            .to_string();

            current_offset += value_len;

            additional_metadata.push((key, value));
        }

        Ok(TokenMetadata {
            update_authority,
            mint,
            name,
            symbol,
            uri,
            additional_metadata,
        })
    }
}

impl Extension for TokenMetadata {
    const TYPE: ExtensionType = ExtensionType::TokenMetadata;
    const LEN: usize = Self::LEN;
    const BASE_STATE: BaseState = BaseState::Mint;
}
