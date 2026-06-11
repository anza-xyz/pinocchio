use {
    super::{sealed, ExtensionType, ExtensionValue},
    solana_zero_copy::unaligned::Bool,
};

/// Memo transfer extension data for token accounts (1 byte).
///
/// When enabled, transfers into this account must be accompanied by a memo
/// instruction.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MemoTransferExtension {
    pub require_incoming_transfer_memos: Bool,
}

impl MemoTransferExtension {
    pub const LEN: usize = core::mem::size_of::<MemoTransferExtension>();
}

impl sealed::Sealed for MemoTransferExtension {}

// SAFETY: `MemoTransferExtension` is repr(C), contains only a `Bool`
// (repr(transparent) over `u8`), has no padding, and all bit patterns are
// valid.
unsafe impl ExtensionValue for MemoTransferExtension {
    const TYPE: ExtensionType = ExtensionType::MemoTransfer;
}
