use {solana_address::Address, solana_program_error::ProgramError};

/// Zero-copy view into token metadata account data.
///
/// This struct represents the **TLV entry payload** for Token-2022's
/// `TokenMetadata` extension (extension type 19). Callers must first
/// extract the TLV entry data from the account before passing it to
/// [`from_bytes`](Metadata::from_bytes) or
/// [`from_bytes_unchecked`](Metadata::from_bytes_unchecked).
///
/// On-chain data layout (inside the Token-2022 TLV entry):
/// - `[0..32]:   update_authority (Address)`
/// - `[32..64]:  mint (Address)`
/// - `[64..68]:  name_len (u32 LE)`
/// - `[68..68+N]: name (UTF-8)`
/// - `[..+4]:    symbol_len (u32 LE)`
/// - `[..+S]:    symbol (UTF-8)`
/// - `[..+4]:    uri_len (u32 LE)`
/// - `[..+U]:    uri (UTF-8)`
/// - `[..+4]:    pair_count (u32 LE) -- number of key-value pairs`
/// - For each pair:
///   - `key_len (u32 LE) + key (UTF-8)`
///   - `value_len (u32 LE) + value (UTF-8)`
pub struct Metadata<'a> {
    /// Authority that can update the metadata.
    update_authority: &'a Address,

    /// Mint associated with this metadata.
    mint: &'a Address,

    /// Token name (raw bytes, UTF-8 guaranteed by Token-2022 program).
    name: &'a [u8],

    /// Token symbol (raw bytes, UTF-8 guaranteed by Token-2022 program).
    symbol: &'a [u8],

    /// Token URI (raw bytes, UTF-8 guaranteed by Token-2022 program).
    uri: &'a [u8],

    /// Additional metadata (raw key-value pairs).
    additional_metadata: &'a [u8],
}

impl<'a> Metadata<'a> {
    /// Minimum data size: `32 (update_authority) + 32 (mint) + 4 (name_len) + 4
    /// (symbol_len) + 4 (uri_len) + 4 (pair_count) = 80` bytes.
    pub const MIN_LEN: usize = 80;

    const UPDATE_AUTHORITY_OFFSET: usize = 0;
    const MINT_OFFSET: usize = 32;
    const FIRST_VARLEN_OFFSET: usize = 64;

    /// Read a `u32` length prefix at the given byte offset.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `offset..offset+4` is within bounds of
    /// `data` and that the pointer is valid for a 4-byte aligned read.
    #[inline(always)]
    unsafe fn read_len_at(data: &[u8], offset: usize) -> usize {
        u32::from_le_bytes(*(data.as_ptr().add(offset) as *const [u8; 4])) as usize
    }

    /// Read a variable-length field slice starting at `offset`.
    ///
    /// Returns the field bytes and the offset past the field `(length prefix +
    /// data)`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `offset..offset+4` and
    /// `offset+4..offset+4+len` are within bounds of `data`.
    #[inline(always)]
    unsafe fn read_field(data: &'a [u8], offset: usize) -> (&'a [u8], usize) {
        let len = Self::read_len_at(data, offset);
        let start = offset + 4;
        (
            core::slice::from_raw_parts(data.as_ptr().add(start), len),
            start + len,
        )
    }

    /// Create a validated `Metadata` view from raw bytes.
    ///
    /// Validates minimum length and that all declared variable-length field
    /// sizes fit within the data. Does **not** validate UTF-8.
    ///
    /// ## On-chain layout
    ///
    /// All variable-length fields use the on-chain encoding written by the
    /// Token-2022 program. Strings (`name`, `symbol`, `uri`) are stored as
    /// `u32_le byte_length | bytes`. The `additional_metadata` field is
    /// encoded as `u32_le pair_count`, followed by `pair_count` pairs where
    /// each pair is `u32_le key_len | key | u32_le value_len | value`.
    #[inline]
    pub fn from_bytes(data: &'a [u8]) -> Result<Metadata<'a>, ProgramError> {
        if data.len() < Self::MIN_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        // Validate all 3 string fields (name, symbol, uri) — each is u32 byte_len +
        // bytes.
        let mut offset = Self::FIRST_VARLEN_OFFSET;

        for _ in 0..3 {
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

            offset = offset
                .checked_add(4 + field_len)
                .ok_or(ProgramError::InvalidAccountData)?;

            if offset > data.len() {
                return Err(ProgramError::InvalidAccountData);
            }
        }

        // Validate that the pair_count prefix (u32) fits within the data.
        if offset
            .checked_add(4)
            .ok_or(ProgramError::InvalidAccountData)?
            > data.len()
        {
            return Err(ProgramError::InvalidAccountData);
        }

        // All bounds are validated; delegate construction to the unchecked path.
        Ok(unsafe { Self::from_bytes_unchecked(data) })
    }

    /// Create a `Metadata` view without any validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `data` contains a valid token metadata
    /// layout with all declared field lengths fitting within the slice.
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(data: &'a [u8]) -> Self {
        let (name, offset) = Self::read_field(data, Self::FIRST_VARLEN_OFFSET);
        let (symbol, offset) = Self::read_field(data, offset);
        let (uri, offset) = Self::read_field(data, offset);
        // Skip the additional_metadata pair count (u32), capture remaining bytes.
        let additional_metadata = &data[offset + 4..];

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

    /// Return the authority that can update the metadata.
    #[inline(always)]
    pub fn update_authority(&self) -> &Address {
        self.update_authority
    }

    /// Return the mint associated with this metadata.
    #[inline(always)]
    pub fn mint(&self) -> &Address {
        self.mint
    }

    /// Token name as raw bytes.
    #[inline(always)]
    pub fn name(&self) -> &[u8] {
        self.name
    }

    /// Token symbol as raw bytes.
    #[inline(always)]
    pub fn symbol(&self) -> &[u8] {
        self.symbol
    }

    /// Token URI as raw bytes.
    #[inline(always)]
    pub fn uri(&self) -> &[u8] {
        self.uri
    }

    /// Raw additional metadata bytes (key-value pairs without the pair count
    /// prefix).
    #[inline(always)]
    pub fn additional_metadata(&self) -> &[u8] {
        self.additional_metadata
    }

    /// Return the token name as a UTF-8 string.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the underlying bytes contain valid UTF-8.
    /// Data written by the Token-2022 program is guaranteed to be valid UTF-8.
    #[inline(always)]
    pub unsafe fn name_as_str_unchecked(&self) -> &str {
        core::str::from_utf8_unchecked(self.name)
    }

    /// Return the token symbol as a UTF-8 string.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the underlying bytes contain valid UTF-8.
    /// Data written by the Token-2022 program is guaranteed to be valid UTF-8.
    #[inline(always)]
    pub unsafe fn symbol_as_str_unchecked(&self) -> &str {
        core::str::from_utf8_unchecked(self.symbol)
    }

    /// Return the token URI as a UTF-8 string.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the underlying bytes contain valid UTF-8.
    /// Data written by the Token-2022 program is guaranteed to be valid UTF-8.
    #[inline(always)]
    pub unsafe fn uri_as_str_unchecked(&self) -> &str {
        core::str::from_utf8_unchecked(self.uri)
    }

    /// Returns an iterator over the additional metadata key-value pairs.
    ///
    /// Each pair is serialized on-chain as:
    /// - `key_len (u32 LE) + key bytes (UTF-8)`
    /// - `value_len (u32 LE) + value bytes (UTF-8)`
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
        // Write pair count (u32 LE), then the serialized pairs.
        bytes.extend_from_slice(&(additional_metadata.len() as u32).to_le_bytes());
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

        assert_eq!(
            metadata.update_authority(),
            &Address::from(update_authority)
        );
        assert_eq!(metadata.mint(), &Address::from(mint));
        assert_eq!(unsafe { metadata.name_as_str_unchecked() }, name);
        assert_eq!(unsafe { metadata.symbol_as_str_unchecked() }, symbol);
        assert_eq!(unsafe { metadata.uri_as_str_unchecked() }, uri);

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

        assert_eq!(bytes.len(), Metadata::MIN_LEN);

        let metadata = Metadata::from_bytes(&bytes).unwrap();

        assert_eq!(
            metadata.update_authority(),
            &Address::from(update_authority)
        );
        assert_eq!(metadata.mint(), &Address::from(mint));
        assert_eq!(unsafe { metadata.name_as_str_unchecked() }, "");
        assert_eq!(unsafe { metadata.symbol_as_str_unchecked() }, "");
        assert_eq!(unsafe { metadata.uri_as_str_unchecked() }, "");
        assert_eq!(metadata.additional_metadata(), &[] as &[u8]);
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

        assert_eq!(checked.update_authority(), unchecked.update_authority());
        assert_eq!(checked.mint(), unchecked.mint());
        assert_eq!(checked.name(), unchecked.name());
        assert_eq!(checked.symbol(), unchecked.symbol());
        assert_eq!(checked.uri(), unchecked.uri());
        assert_eq!(
            checked.additional_metadata(),
            unchecked.additional_metadata()
        );
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

        assert_eq!(
            metadata.update_authority(),
            &Address::from(update_authority)
        );
        assert_eq!(unsafe { metadata.name_as_str_unchecked() }, "TK");
        assert_eq!(unsafe { metadata.symbol_as_str_unchecked() }, "T");
        assert_eq!(unsafe { metadata.uri_as_str_unchecked() }, "https://t");
        // trailing data is captured as additional_metadata (but yields no valid pairs)
        assert_eq!(metadata.additional_metadata_iter().count(), 0);
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
        // Build metadata bytes manually: after name/symbol/uri,
        // write a pair_count then one valid pair + one truncated pair.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0u8; 64]); // authority + mint
        for _ in 0..3 {
            bytes.extend_from_slice(&0u32.to_le_bytes()); // name, symbol, uri
                                                          // (empty)
        }
        // pair_count = 2 (but only the first pair is complete)
        bytes.extend_from_slice(&2u32.to_le_bytes());
        // First pair (complete): key="key", value="val"
        bytes.extend_from_slice(&3u32.to_le_bytes());
        bytes.extend_from_slice(b"key");
        bytes.extend_from_slice(&3u32.to_le_bytes());
        bytes.extend_from_slice(b"val");
        // Truncated second pair (key_len present but no data)
        bytes.extend_from_slice(&10u32.to_le_bytes());

        let metadata = Metadata::from_bytes(&bytes).unwrap();
        let kv: Vec<_> = metadata.additional_metadata_iter().collect();
        // Only the first complete pair is yielded; iterator stops at truncation
        assert_eq!(kv.len(), 1);
        assert_eq!(kv[0], (b"key" as &[u8], b"val" as &[u8]));
    }
}
