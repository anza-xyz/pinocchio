use {
    super::{sealed, ExtensionType, ExtensionValue},
    solana_zero_copy::unaligned::U64,
};

/// Transfer fee amount extension data for token accounts (8 bytes).
///
/// Tracks the fees withheld on this account from inbound transfers governed
/// by the mint's `TransferFeeConfig`. Pending fees accumulate here until the
/// authority harvests them back to the mint.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TransferFeeAmountExtension {
    pub withheld_amount: U64,
}

impl TransferFeeAmountExtension {
    pub const LEN: usize = core::mem::size_of::<TransferFeeAmountExtension>();
}

impl sealed::Sealed for TransferFeeAmountExtension {}

// SAFETY: `TransferFeeAmountExtension` is repr(C), contains only a
// `U64` (repr(transparent) over `[u8; 8]`), has no padding, and all bit
// patterns are valid.
unsafe impl ExtensionValue for TransferFeeAmountExtension {
    const TYPE: ExtensionType = ExtensionType::TransferFeeAmount;
}
