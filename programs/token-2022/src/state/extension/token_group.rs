use {
    super::{sealed, ExtensionType, ExtensionValue},
    solana_address::Address,
    solana_nullable::MaybeNull,
    solana_zero_copy::unaligned::U64,
};

/// Token group extension data for mints (80 bytes).
///
/// Holds the configuration of a group: the authority permitted to update it,
/// the mint it describes, and its current and maximum number of members.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TokenGroupExtension {
    pub update_authority: MaybeNull<Address>,
    pub mint: Address,
    pub size: U64,
    pub max_size: U64,
}

impl TokenGroupExtension {
    pub const LEN: usize = core::mem::size_of::<TokenGroupExtension>();
}

impl sealed::Sealed for TokenGroupExtension {}

// SAFETY: `TokenGroupExtension` is repr(C), contains only `MaybeNull<Address>`
// and `Address` fields (`[u8; 32]`) and `U64` fields (repr(transparent) over
// `[u8; 8]`), has no padding, and all bit patterns are valid.
unsafe impl ExtensionValue for TokenGroupExtension {
    const TYPE: ExtensionType = ExtensionType::TokenGroup;
}
