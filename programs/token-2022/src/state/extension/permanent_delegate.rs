use {
    super::{sealed, ExtensionType, ExtensionValue},
    solana_address::Address,
    solana_nullable::MaybeNull,
};

/// Permanent delegate extension data for mints (32 bytes).
///
/// When set on a mint, the delegate has unrestricted transfer and
/// burn authority over all token accounts for the mint.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PermanentDelegateExtension {
    pub delegate: MaybeNull<Address>,
}

impl PermanentDelegateExtension {
    pub const LEN: usize = core::mem::size_of::<PermanentDelegateExtension>();
}

impl sealed::Sealed for PermanentDelegateExtension {}

// SAFETY: `PermanentDelegateExtension` is repr(C), contains only
// `MaybeNull<Address>` which is repr(transparent) over `Address` (`[u8; 32]`),
// has no padding, and all bit patterns are valid.
unsafe impl ExtensionValue for PermanentDelegateExtension {
    const TYPE: ExtensionType = ExtensionType::PermanentDelegate;
}
