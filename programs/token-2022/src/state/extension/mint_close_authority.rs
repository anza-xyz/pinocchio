use {
    super::{sealed, ExtensionType, ExtensionValue},
    solana_address::Address,
    solana_nullable::MaybeNull,
};

/// Mint close authority extension data for mints (32 bytes).
///
/// When set on a mint, the authority is permitted to close the mint account
/// once its supply is zero.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MintCloseAuthorityExtension {
    pub close_authority: MaybeNull<Address>,
}

impl MintCloseAuthorityExtension {
    pub const LEN: usize = core::mem::size_of::<MintCloseAuthorityExtension>();
}

impl sealed::Sealed for MintCloseAuthorityExtension {}

// SAFETY: `MintCloseAuthorityExtension` is repr(C), contains only
// `MaybeNull<Address>` which is repr(transparent) over `Address` (`[u8; 32]`),
// has no padding, and all bit patterns are valid.
unsafe impl ExtensionValue for MintCloseAuthorityExtension {
    const TYPE: ExtensionType = ExtensionType::MintCloseAuthority;
}
