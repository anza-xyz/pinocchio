use {
    super::{sealed, Extension, ExtensionType, ExtensionValue},
    solana_zero_copy::unaligned::Bool,
};

/// Transfer hook account extension data (1 byte).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TransferHookAccountExtension {
    pub transferring: Bool,
}

impl TransferHookAccountExtension {
    pub const LEN: usize = core::mem::size_of::<TransferHookAccountExtension>();
}

// SAFETY: `TransferHookAccountExtension` is repr(C), contains only `Bool`
// which is `repr(transparent)` over `u8`, has no padding, and all bit
// patterns are valid.
impl sealed::SealedExtension for TransferHookAccountExtension {}
unsafe impl Extension for TransferHookAccountExtension {}

impl ExtensionValue for TransferHookAccountExtension {
    const TYPE: ExtensionType = ExtensionType::TransferHookAccount;
}
