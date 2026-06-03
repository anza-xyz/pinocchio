use {
    super::{sealed, ExtensionType, ExtensionValue},
    solana_address::Address,
    solana_nullable::MaybeNull,
};

/// Group pointer extension data for mints (64 bytes).
///
/// Points to the account that holds the mint's token group configuration and
/// names the authority permitted to update that pointer.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GroupPointerExtension {
    pub authority: MaybeNull<Address>,
    pub group_address: MaybeNull<Address>,
}

impl GroupPointerExtension {
    pub const LEN: usize = core::mem::size_of::<GroupPointerExtension>();
}

impl sealed::Sealed for GroupPointerExtension {}

// SAFETY: `GroupPointerExtension` is repr(C), contains only
// `MaybeNull<Address>` fields which are repr(transparent) over `Address`
// (`[u8; 32]`), has no padding, and all bit patterns are valid.
unsafe impl ExtensionValue for GroupPointerExtension {
    const TYPE: ExtensionType = ExtensionType::GroupPointer;
}
