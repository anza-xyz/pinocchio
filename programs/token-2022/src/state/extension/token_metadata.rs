use {
    super::{sealed, ExtensionType, VariableLenExtension},
    solana_address::Address,
    solana_nullable::MaybeNull,
    solana_program_error::ProgramError,
};

/// Borrowed view of the token metadata extension data for mints.
///
/// Unlike fixed-size extensions, the metadata is variable-length
/// Borsh-encoded data, so it is parsed and validated when the view is
/// created instead of being cast in place.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TokenMetadataExtension<'a> {
    pub update_authority: &'a MaybeNull<Address>,
    pub mint: &'a Address,
    pub name: &'a str,
    pub symbol: &'a str,
    pub uri: &'a str,
    additional_metadata_len: u32,
    additional_metadata: &'a [u8],
}

impl<'a> TokenMetadataExtension<'a> {
    /// Parse the token metadata from the extension value bytes.
    #[inline]
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ProgramError> {
        if data.len() < 64 {
            return Err(ProgramError::InvalidAccountData);
        }
        let (update_authority, rest) = data.split_at(32);
        let (mint, mut rest) = rest.split_at(32);

        let name = read_str(&mut rest)?;
        let symbol = read_str(&mut rest)?;
        let uri = read_str(&mut rest)?;
        let additional_metadata_len = read_u32(&mut rest)?;
        let additional_metadata = rest;

        for _ in 0..additional_metadata_len {
            read_str(&mut rest)?;
            read_str(&mut rest)?;
        }

        Ok(Self {
            // SAFETY: `MaybeNull<Address>` is repr(transparent) over `[u8; 32]`,
            // so any 32 bytes are a valid, alignment-1 value.
            update_authority: unsafe { &*(update_authority.as_ptr() as *const MaybeNull<Address>) },
            // SAFETY: `Address` is repr(transparent) over `[u8; 32]`.
            mint: unsafe { &*(mint.as_ptr() as *const Address) },
            name,
            symbol,
            uri,
            additional_metadata_len,
            additional_metadata,
        })
    }

    /// Iterate over the additional `(key, value)` metadata pairs.
    #[inline]
    pub fn additional_metadata(&self) -> impl Iterator<Item = (&'a str, &'a str)> {
        let mut rest = self.additional_metadata;
        (0..self.additional_metadata_len).map_while(move |_| {
            let key = read_str(&mut rest).ok()?;
            let value = read_str(&mut rest).ok()?;
            Some((key, value))
        })
    }
}

impl sealed::Sealed for TokenMetadataExtension<'_> {}

impl<'a> VariableLenExtension<'a> for TokenMetadataExtension<'a> {
    const TYPE: ExtensionType = ExtensionType::TokenMetadata;

    #[inline]
    fn from_bytes(data: &'a [u8]) -> Result<Self, ProgramError> {
        TokenMetadataExtension::from_bytes(data)
    }
}

#[inline(always)]
fn read_u32(data: &mut &[u8]) -> Result<u32, ProgramError> {
    let (len, rest) = data
        .split_first_chunk::<4>()
        .ok_or(ProgramError::InvalidAccountData)?;
    *data = rest;
    Ok(u32::from_le_bytes(*len))
}

#[inline(always)]
fn read_str<'a>(data: &mut &'a [u8]) -> Result<&'a str, ProgramError> {
    let len = read_u32(data)? as usize;
    if data.len() < len {
        return Err(ProgramError::InvalidAccountData);
    }
    let (value, rest) = data.split_at(len);
    *data = rest;
    core::str::from_utf8(value).map_err(|_| ProgramError::InvalidAccountData)
}
