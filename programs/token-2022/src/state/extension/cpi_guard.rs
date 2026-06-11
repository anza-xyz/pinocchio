use {
    super::{sealed, ExtensionType, ExtensionValue},
    solana_zero_copy::unaligned::Bool,
};

/// CPI guard extension data for token accounts (1 byte).
///
/// When enabled, privileged token operations on this account are blocked from
/// executing via cross-program invocation.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CpiGuardExtension {
    pub lock_cpi: Bool,
}

impl CpiGuardExtension {
    pub const LEN: usize = core::mem::size_of::<CpiGuardExtension>();
}

impl sealed::Sealed for CpiGuardExtension {}

// SAFETY: `CpiGuardExtension` is repr(C), contains only a `Bool`
// (repr(transparent) over `u8`), has no padding, and all bit patterns are
// valid.
unsafe impl ExtensionValue for CpiGuardExtension {
    const TYPE: ExtensionType = ExtensionType::CpiGuard;
}
