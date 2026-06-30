use {
    super::{sealed, ExtensionType, ExtensionValue},
    solana_address::Address,
    solana_nullable::MaybeNull,
};

/// Metadata pointer extension data for mints (64 bytes).
///
/// Points to the account that holds the mint's token metadata and names the
/// authority permitted to update that pointer.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MetadataPointerExtension {
    pub authority: MaybeNull<Address>,
    pub metadata_address: MaybeNull<Address>,
}

impl MetadataPointerExtension {
    pub const LEN: usize = core::mem::size_of::<MetadataPointerExtension>();
}

impl sealed::Sealed for MetadataPointerExtension {}

// SAFETY: `MetadataPointerExtension` is repr(C), contains only
// `MaybeNull<Address>` fields which are repr(transparent) over `Address`
// (`[u8; 32]`), has no padding, and all bit patterns are valid.
unsafe impl ExtensionValue for MetadataPointerExtension {
    const TYPE: ExtensionType = ExtensionType::MetadataPointer;
}
