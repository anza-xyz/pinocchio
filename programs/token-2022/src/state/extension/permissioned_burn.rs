use {
    super::{sealed, ExtensionType, ExtensionValue},
    solana_address::Address,
    solana_nullable::MaybeNull,
};

/// Permissioned burn extension data for mints (32 bytes).
///
/// When set on a mint, the authority is required for burning tokens from
/// accounts for the mint.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PermissionedBurnExtension {
    pub authority: MaybeNull<Address>,
}

impl PermissionedBurnExtension {
    pub const LEN: usize = core::mem::size_of::<PermissionedBurnExtension>();
}

impl sealed::Sealed for PermissionedBurnExtension {}

// SAFETY: `PermissionedBurnExtension` is repr(C), contains only
// `MaybeNull<Address>` which is repr(transparent) over `Address` (`[u8; 32]`),
// has no padding, and all bit patterns are valid.
unsafe impl ExtensionValue for PermissionedBurnExtension {
    const TYPE: ExtensionType = ExtensionType::PermissionedBurn;
}
