use super::{sealed, ExtensionType, ExtensionValue};

/// Non-transferable extension (marker, 0 bytes).
///
/// When set on a mint, tokens for the mint cannot be transferred between
/// accounts.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NonTransferableExtension;

impl NonTransferableExtension {
    pub const LEN: usize = core::mem::size_of::<NonTransferableExtension>();
}

impl sealed::Sealed for NonTransferableExtension {}

// SAFETY: `NonTransferableExtension` is a zero-sized type with no
// representation requirements.
unsafe impl ExtensionValue for NonTransferableExtension {
    const TYPE: ExtensionType = ExtensionType::NonTransferable;
}
