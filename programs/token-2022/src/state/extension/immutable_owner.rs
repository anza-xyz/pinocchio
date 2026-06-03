use super::{sealed, ExtensionType, ExtensionValue};

/// Immutable owner extension (marker, 0 bytes).
///
/// Indicates that the token account's owner authority cannot be changed.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImmutableOwnerExtension;

impl ImmutableOwnerExtension {
    pub const LEN: usize = core::mem::size_of::<ImmutableOwnerExtension>();
}

impl sealed::Sealed for ImmutableOwnerExtension {}

// SAFETY: `ImmutableOwnerExtension` is a zero-sized type with no
// representation requirements.
unsafe impl ExtensionValue for ImmutableOwnerExtension {
    const TYPE: ExtensionType = ExtensionType::ImmutableOwner;
}
