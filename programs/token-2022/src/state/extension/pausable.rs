use {
    super::{sealed, ExtensionType, ExtensionValue},
    solana_address::Address,
    solana_nullable::MaybeNull,
    solana_zero_copy::unaligned::Bool,
};

/// Pausable extension data for mints (33 bytes).
///
/// When set on a mint, the authority can pause and resume minting,
/// transferring, and burning of tokens for the mint.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PausableExtension {
    pub authority: MaybeNull<Address>,
    pub paused: Bool,
}

impl PausableExtension {
    pub const LEN: usize = core::mem::size_of::<PausableExtension>();
}

impl sealed::Sealed for PausableExtension {}

// SAFETY: `PausableExtension` is repr(C), contains a `MaybeNull<Address>`
// (repr(transparent) over `[u8; 32]`) followed by a `Bool`
// (repr(transparent) over `u8`), has no padding, and all bit patterns are
// valid.
unsafe impl ExtensionValue for PausableExtension {
    const TYPE: ExtensionType = ExtensionType::Pausable;
}
