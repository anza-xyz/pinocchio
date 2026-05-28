use super::{sealed, ExtensionType, ExtensionValue};

/// Non-transferable account extension (marker, 0 bytes).
///
/// Required on token accounts of mints with the `NonTransferable` extension.
/// The mint also forces `ImmutableOwner` on these accounts so the marker
/// can't be bypassed.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NonTransferableAccountExtension;

impl NonTransferableAccountExtension {
    pub const LEN: usize = core::mem::size_of::<NonTransferableAccountExtension>();
}

impl sealed::Sealed for NonTransferableAccountExtension {}

// SAFETY: `NonTransferableAccountExtension` is a zero-sized type with
// no representation requirements.
unsafe impl ExtensionValue for NonTransferableAccountExtension {
    const TYPE: ExtensionType = ExtensionType::NonTransferableAccount;
}
