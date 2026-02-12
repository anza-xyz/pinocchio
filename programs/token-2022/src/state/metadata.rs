use solana_account_view::AccountView;
use solana_address::Address;
use solana_program_error::ProgramError;

use crate::ID;

/// Zero-copy view into token metadata account data.
///
/// On-chain data layout:
/// - `[0..32]`:   `update_authority` (`Address`)
/// - `[32..64]`:  mint (`Address`)
/// - `[64..68]`:  `name_len` (`u32` LE)
/// - `[68..68+N]`: name (UTF-8)
/// - `[..+4]`:    `symbol_len` (`u32` LE)
/// - `[..+S]`:    symbol (UTF-8)
/// - `[..+4]`:    `uri_len` (`u32` LE)
/// - `[..+U]`:    uri (UTF-8)
/// - `[..+4]`:    `additional_metadata_len` (`u32` LE)
/// - `[..+A]`:    additional metadata
pub struct Metadata<'a> {
    /// Authority that can update the metadata.
    pub update_authority: &'a Address,

    /// Mint associated with this metadata.
    pub mint: &'a Address,

    /// Token name (raw bytes, UTF-8 guaranteed by Token-2022 program).
    pub name: &'a [u8],

    /// Token symbol (raw bytes, UTF-8 guaranteed by Token-2022 program).
    pub symbol: &'a [u8],

    /// Token URI (raw bytes, UTF-8 guaranteed by Token-2022 program).
    pub uri: &'a [u8],

    /// Additional metadata (raw key-value pairs).
    pub additional_metadata: &'a [u8],
}

impl<'a> Metadata<'a> {
    /// Minimum account data size: `32 + 32 + 4*4 = 80` bytes.
    pub const MIN_SIZE: usize = 80;

    const UPDATE_AUTHORITY_OFFSET: usize = 0;
    const MINT_OFFSET: usize = 32;
    const FIRST_VARLEN_OFFSET: usize = 64;

    /// Read a `u32` length prefix at the given byte offset.
    #[inline(always)]
    unsafe fn read_len_at(data: &[u8], offset: usize) -> usize {
        u32::from_le_bytes(*(data.as_ptr().add(offset) as *const [u8; 4])) as usize
    }

    /// Read a variable-length field slice starting at `offset`.
    ///
    /// Returns the field bytes and the offset past the field `(length prefix + data)`.
    #[inline(always)]
    unsafe fn read_field(data: &'a [u8], offset: usize) -> (&'a [u8], usize) {
        let len = Self::read_len_at(data, offset);
        let start = offset + 4;
        (
            core::slice::from_raw_parts(data.as_ptr().add(start), len),
            start + len,
        )
    }

    /// Return a `Metadata` from the given account view.
    ///
    /// This method performs owner and length validation on `AccountView`,
    /// and validates that all declared field lengths fit within the data.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to borrow the account data (e.g., there are
    /// no mutable borrows of the account data).
    #[inline]
    pub unsafe fn from_account_view_unchecked(
        account_view: &'a AccountView,
    ) -> Result<Metadata<'a>, ProgramError> {
        if account_view.data_len() < Self::MIN_SIZE {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_view.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Self::from_bytes(account_view.borrow_unchecked())
    }

    /// Create a validated `Metadata` view from raw bytes.
    ///
    /// Validates minimum length and that all declared variable-length field
    /// sizes fit within the data. Does **not** validate UTF-8.
    pub fn from_bytes(data: &'a [u8]) -> Result<Metadata<'a>, ProgramError> {
        if data.len() < Self::MIN_SIZE {
            return Err(ProgramError::InvalidAccountData);
        }

        // Walk the 4 variable-length fields (name, symbol, uri, additional_metadata)
        // to ensure all declared lengths fit within the data and capture slices.
        let mut offset = Self::FIRST_VARLEN_OFFSET;
        let mut fields: [&[u8]; 4] = [&[]; 4];

        for field in &mut fields {
            if offset
                .checked_add(4)
                .ok_or(ProgramError::InvalidAccountData)?
                > data.len()
            {
                return Err(ProgramError::InvalidAccountData);
            }

            let field_len = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;

            let start = offset + 4;

            offset = offset
                .checked_add(4 + field_len)
                .ok_or(ProgramError::InvalidAccountData)?;

            if offset > data.len() {
                return Err(ProgramError::InvalidAccountData);
            }

            *field = &data[start..start + field_len];
        }

        Ok(Metadata {
            update_authority: unsafe {
                &*(data.as_ptr().add(Self::UPDATE_AUTHORITY_OFFSET) as *const Address)
            },
            mint: unsafe { &*(data.as_ptr().add(Self::MINT_OFFSET) as *const Address) },
            name: fields[0],
            symbol: fields[1],
            uri: fields[2],
            additional_metadata: fields[3],
        })
    }

    /// Create a `Metadata` view without any validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `data` contains a valid token metadata
    /// layout with all declared field lengths fitting within the slice.
    /// The Token-2022 program guarantees valid UTF-8 for name, symbol, and
    /// uri fields, so data from a valid account is safe to read as `&str`.
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(data: &'a [u8]) -> Self {
        let (name, offset) = Self::read_field(data, Self::FIRST_VARLEN_OFFSET);
        let (symbol, offset) = Self::read_field(data, offset);
        let (uri, offset) = Self::read_field(data, offset);
        let (additional_metadata, _) = Self::read_field(data, offset);

        Metadata {
            update_authority: &*(data.as_ptr().add(Self::UPDATE_AUTHORITY_OFFSET)
                as *const Address),
            mint: &*(data.as_ptr().add(Self::MINT_OFFSET) as *const Address),
            name,
            symbol,
            uri,
            additional_metadata,
        }
    }

    /// Return the token name as a UTF-8 string.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the underlying bytes contain valid UTF-8.
    /// Data written by the Token-2022 program is guaranteed to be valid UTF-8.
    #[inline(always)]
    pub unsafe fn name_as_str(&self) -> &str {
        core::str::from_utf8_unchecked(self.name)
    }

    /// Return the token symbol as a UTF-8 string.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the underlying bytes contain valid UTF-8.
    /// Data written by the Token-2022 program is guaranteed to be valid UTF-8.
    #[inline(always)]
    pub unsafe fn symbol_as_str(&self) -> &str {
        core::str::from_utf8_unchecked(self.symbol)
    }

    /// Return the token URI as a UTF-8 string.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the underlying bytes contain valid UTF-8.
    /// Data written by the Token-2022 program is guaranteed to be valid UTF-8.
    #[inline(always)]
    pub unsafe fn uri_as_str(&self) -> &str {
        core::str::from_utf8_unchecked(self.uri)
    }

    /// Returns an iterator over the additional metadata key-value pairs.
    ///
    /// Each pair is serialized on-chain as:
    /// - `key_len` (`u32` LE) + key bytes (UTF-8)
    /// - `value_len` (`u32` LE) + value bytes (UTF-8)
    ///
    /// The iterator stops when the remaining data is too short to contain
    /// another complete pair.
    #[inline(always)]
    pub fn additional_metadata_iter(&self) -> AdditionalMetadataIterator<'a> {
        AdditionalMetadataIterator {
            data: self.additional_metadata,
            offset: 0,
        }
    }
}

/// Zero-copy iterator over additional metadata key-value pairs.
///
/// Yields `(&[u8], &[u8])` for each (key, value) pair. Stops when the
/// remaining bytes cannot form a complete pair.
pub struct AdditionalMetadataIterator<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for AdditionalMetadataIterator<'a> {
    type Item = (&'a [u8], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let remaining = self.data.len() - self.offset;

        // Need at least 4 (key_len) + 4 (value_len) = 8 bytes for an empty pair.
        if remaining < 8 {
            return None;
        }

        let key_len = u32::from_le_bytes([
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
        ]) as usize;

        let key_start = self.offset + 4;
        let key_end = key_start.checked_add(key_len)?;

        if key_end + 4 > self.data.len() {
            return None;
        }

        let value_len = u32::from_le_bytes([
            self.data[key_end],
            self.data[key_end + 1],
            self.data[key_end + 2],
            self.data[key_end + 3],
        ]) as usize;

        let value_start = key_end + 4;
        let value_end = value_start.checked_add(value_len)?;

        if value_end > self.data.len() {
            return None;
        }

        let key = &self.data[key_start..key_end];
        let value = &self.data[value_start..value_end];
        self.offset = value_end;

        Some((key, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::vec::Vec;

    fn serialize_additional_metadata(pairs: &[(&str, &str)]) -> Vec<u8> {
        let mut buf = Vec::new();
        for (key, value) in pairs {
            buf.extend_from_slice(&(key.len() as u32).to_le_bytes());
            buf.extend_from_slice(key.as_bytes());
            buf.extend_from_slice(&(value.len() as u32).to_le_bytes());
            buf.extend_from_slice(value.as_bytes());
        }
        buf
    }

    fn create_test_metadata_bytes(
        update_authority: &[u8; 32],
        mint: &[u8; 32],
        name: &str,
        symbol: &str,
        uri: &str,
        additional_metadata: &[(&str, &str)],
    ) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(update_authority);
        bytes.extend_from_slice(mint);

        bytes.extend_from_slice(&(name.len() as u32).to_le_bytes());
        bytes.extend_from_slice(name.as_bytes());

        bytes.extend_from_slice(&(symbol.len() as u32).to_le_bytes());
        bytes.extend_from_slice(symbol.as_bytes());

        bytes.extend_from_slice(&(uri.len() as u32).to_le_bytes());
        bytes.extend_from_slice(uri.as_bytes());

        let additional = serialize_additional_metadata(additional_metadata);
        bytes.extend_from_slice(&(additional.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&additional);

        bytes
    }

    #[test]
    fn test_from_bytes_valid() {
        let update_authority = [5u8; 32];
        let mint = [6u8; 32];
        let name = "My Token";
        let symbol = "MTK";
        let uri = "https://metadata.example.com/token.json";
        let pairs = &[("key1", "value1"), ("key2", "value2")];

        let bytes = create_test_metadata_bytes(&update_authority, &mint, name, symbol, uri, pairs);

        let metadata = Metadata::from_bytes(&bytes).unwrap();

        assert_eq!(metadata.update_authority, &Address::from(update_authority));
        assert_eq!(metadata.mint, &Address::from(mint));
        assert_eq!(unsafe { metadata.name_as_str() }, name);
        assert_eq!(unsafe { metadata.symbol_as_str() }, symbol);
        assert_eq!(unsafe { metadata.uri_as_str() }, uri);

        let kv: Vec<_> = metadata.additional_metadata_iter().collect();
        assert_eq!(kv.len(), 2);
        assert_eq!(kv[0], (b"key1" as &[u8], b"value1" as &[u8]));
        assert_eq!(kv[1], (b"key2" as &[u8], b"value2" as &[u8]));
    }

    #[test]
    fn test_from_bytes_too_short() {
        let bytes = [0u8; 79];
        assert!(Metadata::from_bytes(&bytes).is_err());
    }

    #[test]
    fn test_from_bytes_truncated_name() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0u8; 64]);
        // name_len = 100, but only 16 bytes remain (4×4 length prefixes)
        bytes.extend_from_slice(&100u32.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 12]);

        assert!(Metadata::from_bytes(&bytes).is_err());
    }

    #[test]
    fn test_from_bytes_empty_fields() {
        let update_authority = [1u8; 32];
        let mint = [2u8; 32];

        let bytes = create_test_metadata_bytes(&update_authority, &mint, "", "", "", &[]);

        assert_eq!(bytes.len(), Metadata::MIN_SIZE);

        let metadata = Metadata::from_bytes(&bytes).unwrap();

        assert_eq!(metadata.update_authority, &Address::from(update_authority));
        assert_eq!(metadata.mint, &Address::from(mint));
        assert_eq!(unsafe { metadata.name_as_str() }, "");
        assert_eq!(unsafe { metadata.symbol_as_str() }, "");
        assert_eq!(unsafe { metadata.uri_as_str() }, "");
        assert_eq!(metadata.additional_metadata, &[] as &[u8]);
        assert_eq!(metadata.additional_metadata_iter().count(), 0);
    }

    #[test]
    fn test_from_bytes_overflow_len() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0u8; 64]);
        // name_len = u32::MAX → checked_add should catch the overflow
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 12]);

        assert!(Metadata::from_bytes(&bytes).is_err());
    }

    #[test]
    fn test_from_bytes_unchecked_matches_from_bytes() {
        let update_authority = [7u8; 32];
        let mint = [8u8; 32];
        let name = "Unchecked Token";
        let symbol = "UCK";
        let uri = "https://example.com/uck.json";
        let pairs = &[("extra", "data")];

        let bytes = create_test_metadata_bytes(&update_authority, &mint, name, symbol, uri, pairs);

        let checked = Metadata::from_bytes(&bytes).unwrap();
        let unchecked = unsafe { Metadata::from_bytes_unchecked(&bytes) };

        assert_eq!(checked.update_authority, unchecked.update_authority);
        assert_eq!(checked.mint, unchecked.mint);
        assert_eq!(checked.name, unchecked.name);
        assert_eq!(checked.symbol, unchecked.symbol);
        assert_eq!(checked.uri, unchecked.uri);
        assert_eq!(checked.additional_metadata, unchecked.additional_metadata);
    }

    #[test]
    fn test_from_bytes_trailing_data() {
        let update_authority = [3u8; 32];
        let mint = [4u8; 32];

        let mut bytes =
            create_test_metadata_bytes(&update_authority, &mint, "TK", "T", "https://t", &[]);

        // Append trailing bytes (e.g. token-2022 extensions after metadata)
        bytes.extend_from_slice(&[0xFFu8; 64]);

        let metadata = Metadata::from_bytes(&bytes).unwrap();

        assert_eq!(metadata.update_authority, &Address::from(update_authority));
        assert_eq!(unsafe { metadata.name_as_str() }, "TK");
        assert_eq!(unsafe { metadata.symbol_as_str() }, "T");
        assert_eq!(unsafe { metadata.uri_as_str() }, "https://t");
        assert_eq!(metadata.additional_metadata, &[] as &[u8]);
    }

    #[test]
    fn test_additional_metadata_iterator() {
        let pairs = &[
            ("trait_type", "Background"),
            ("value", "Blue"),
            ("display_type", "string"),
        ];
        let bytes = create_test_metadata_bytes(&[0u8; 32], &[0u8; 32], "", "", "", pairs);
        let metadata = Metadata::from_bytes(&bytes).unwrap();

        let kv: Vec<_> = metadata.additional_metadata_iter().collect();
        assert_eq!(kv.len(), 3);
        assert_eq!(kv[0], (b"trait_type" as &[u8], b"Background" as &[u8]));
        assert_eq!(kv[1], (b"value" as &[u8], b"Blue" as &[u8]));
        assert_eq!(kv[2], (b"display_type" as &[u8], b"string" as &[u8]));
    }

    #[test]
    fn test_additional_metadata_iterator_truncated() {
        let mut additional = Vec::new();
        // One valid pair
        additional.extend_from_slice(&3u32.to_le_bytes());
        additional.extend_from_slice(b"key");
        additional.extend_from_slice(&3u32.to_le_bytes());
        additional.extend_from_slice(b"val");
        // Truncated second pair (key_len present but no data)
        additional.extend_from_slice(&10u32.to_le_bytes());

        // Build metadata bytes manually with raw additional_metadata blob
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0u8; 64]); // authority + mint
        for _ in 0..3 {
            bytes.extend_from_slice(&0u32.to_le_bytes()); // name, symbol, uri
        }
        bytes.extend_from_slice(&(additional.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&additional);

        let metadata = Metadata::from_bytes(&bytes).unwrap();
        let kv: Vec<_> = metadata.additional_metadata_iter().collect();
        // Only the first complete pair is yielded
        assert_eq!(kv.len(), 1);
        assert_eq!(kv[0], (b"key" as &[u8], b"val" as &[u8]));
    }
}
