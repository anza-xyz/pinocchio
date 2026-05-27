use {
    super::{sealed, ExtensionType, ExtensionValue},
    solana_address::Address,
    solana_nullable::MaybeNull,
};

/// Transfer hook extension data for mints (64 bytes).
///
/// Configures a custom program to execute additional logic on every
/// transfer involving this mint.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransferHookExtension {
    pub authority: MaybeNull<Address>,
    pub program_id: MaybeNull<Address>,
}

impl TransferHookExtension {
    pub const LEN: usize = core::mem::size_of::<TransferHookExtension>();
}

impl sealed::Sealed for TransferHookExtension {}

// SAFETY: `TransferHookExtension` is repr(C), contains only
// `MaybeNull<Address>` fields which are repr(transparent) over `Address`
// (`[u8; 32]`), has no padding, and all bit patterns are valid.
unsafe impl ExtensionValue for TransferHookExtension {
    const TYPE: ExtensionType = ExtensionType::TransferHook;
}
