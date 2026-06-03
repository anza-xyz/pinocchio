use super::{sealed, ExtensionType, ExtensionValue};

/// Pausable account extension (marker, 0 bytes).
///
/// Required on token accounts of mints with the `Pausable` extension.
/// Transfers, mints, and burns on this account are rejected while
/// the mint is paused.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PausableAccountExtension;

impl PausableAccountExtension {
    pub const LEN: usize = core::mem::size_of::<PausableAccountExtension>();
}

impl sealed::Sealed for PausableAccountExtension {}

// SAFETY: `PausableAccountExtension` is a zero-sized type with no
// representation requirements.
unsafe impl ExtensionValue for PausableAccountExtension {
    const TYPE: ExtensionType = ExtensionType::PausableAccount;
}
