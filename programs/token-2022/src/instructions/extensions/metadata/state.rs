use core::str;

use crate::ID;
use solana_account_view::AccountView;
use solana_address::{Address, ADDRESS_BYTES};
use solana_program_error::ProgramError;

/// State for Metadata for a token
#[repr(C)]
pub struct TokenMetadata<'a> {
    /// The authority that can sign to update the metadata
    pub update_authority: Address,
    /// The associated mint, used to counter spoofing to be sure that metadata
    /// belongs to a particular mint
    pub mint: Address,
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
    /// The minimum size of the metadata account: 80 bytes
    /// [32 (update_authority) + 32 (mint) + 4 (name_len) + 4 (symbol_len) + 4 (uri_len) + 4 (additional_metadata_len)]
    pub const MIN_SIZE: usize = 80;

    /// Return a `TokenMetadata` from the given account view.
    ///
    /// This method performs owner and length validation on `AccountView`, safe borrowing
    /// the account data.
    #[inline]
    pub unsafe fn from_account_view<'a>(
        account_view: &'a AccountView,
    ) -> Result<TokenMetadata<'a>, ProgramError> {
        if account_view.data_len() < Self::MIN_SIZE {
            return Err(ProgramError::InvalidAccountData);
        }

        if !account_view.owned_by(&ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Self::from_bytes_unchecked(account_view.borrow_unchecked())
    }

    /// Parse a `TokenMetadata` from the given bytes.
    ///
    /// This method validates the data length and parses the variable-length fields.
    pub unsafe fn from_bytes_unchecked<'a>(
        data: &'a [u8],
    ) -> Result<TokenMetadata<'a>, ProgramError> {
        if data.len() < Self::MIN_SIZE {
            return Err(ProgramError::InvalidAccountData);
        }

        let mut offset: usize = 0;

        let update_authority = unsafe { core::ptr::read(data.as_ptr() as *const Address) };

        offset += ADDRESS_BYTES;

        let mint = unsafe { core::ptr::read(data.as_ptr().add(offset) as *const Address) };

        offset += ADDRESS_BYTES;

        let name_len =
            u32::from_le_bytes(unsafe { *(data.as_ptr().add(offset) as *const [u8; 4]) });

        offset += 4;

        let name_bytes =
            unsafe { core::slice::from_raw_parts(data.as_ptr().add(offset), name_len as usize) };
        let name =
            core::str::from_utf8(name_bytes).map_err(|_| ProgramError::InvalidAccountData)?;

        offset += name_len as usize;

        let symbol_len =
            u32::from_le_bytes(unsafe { *(data.as_ptr().add(offset) as *const [u8; 4]) });

        offset += 4;

        let symbol_bytes =
            unsafe { core::slice::from_raw_parts(data.as_ptr().add(offset), symbol_len as usize) };
        let symbol =
            core::str::from_utf8(symbol_bytes).map_err(|_| ProgramError::InvalidAccountData)?;

        offset += symbol_len as usize;

        let uri_len = u32::from_le_bytes(unsafe { *(data.as_ptr().add(offset) as *const [u8; 4]) });

        offset += 4;

        let uri_bytes =
            unsafe { core::slice::from_raw_parts(data.as_ptr().add(offset), uri_len as usize) };
        let uri = core::str::from_utf8(uri_bytes).map_err(|_| ProgramError::InvalidAccountData)?;

        offset += uri_len as usize;

        let additional_metadata_len =
            u32::from_le_bytes(unsafe { *(data.as_ptr().add(offset) as *const [u8; 4]) });

        offset += 4;

        let additional_metadata =
            unsafe { core::slice::from_raw_parts(data.as_ptr().add(offset), data.len() - offset) };

        Ok(TokenMetadata {
            update_authority,
            mint,
            name_len,
            name,
            symbol_len,
            symbol,
            uri_len,
            uri,
            additional_metadata_len,
            additional_metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::vec::Vec;

    fn create_test_metadata_bytes(
        update_authority: &[u8; 32],
        mint: &[u8; 32],
        name: &str,
        symbol: &str,
        uri: &str,
        additional_metadata: &[u8],
    ) -> Vec<u8> {
        let mut bytes = Vec::new();

        // update_authority
        bytes.extend_from_slice(update_authority);

        // mint
        bytes.extend_from_slice(mint);

        // name_len + name
        bytes.extend_from_slice(&(name.len() as u32).to_le_bytes());
        bytes.extend_from_slice(name.as_bytes());

        // symbol_len + symbol
        bytes.extend_from_slice(&(symbol.len() as u32).to_le_bytes());
        bytes.extend_from_slice(symbol.as_bytes());

        // uri_len + uri
        bytes.extend_from_slice(&(uri.len() as u32).to_le_bytes());
        bytes.extend_from_slice(uri.as_bytes());

        // additional_metadata_len + additional_metadata
        bytes.extend_from_slice(&(additional_metadata.len() as u32).to_le_bytes());
        bytes.extend_from_slice(additional_metadata);

        bytes
    }

    #[test]
    fn test_from_bytes_with_additional_metadata() {
        let update_authority = [5u8; 32];
        let mint = [6u8; 32];
        let name = "My Token";
        let symbol = "MTK";
        let uri = "https://metadata.example.com/token.json";
        let additional_metadata = b"key1:value1;key2:value2";

        let bytes = create_test_metadata_bytes(
            &update_authority,
            &mint,
            name,
            symbol,
            uri,
            additional_metadata,
        );

        let metadata = unsafe { TokenMetadata::from_bytes_unchecked(&bytes).unwrap() };

        assert_eq!(metadata.update_authority, update_authority.into());
        assert_eq!(metadata.mint, mint.into());
        assert_eq!(metadata.name, name);
        assert_eq!(metadata.symbol, symbol);
        assert_eq!(metadata.uri, uri);
        assert_eq!(metadata.additional_metadata, additional_metadata);
    }
}
