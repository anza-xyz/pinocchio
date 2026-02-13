use super::{sealed, ExtensionType, ExtensionValue, Pod};

/// Transfer hook account extension data (1 byte).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TransferHookAccountExtension {
    transferring: u8,
}

impl TransferHookAccountExtension {
    pub const LEN: usize = core::mem::size_of::<TransferHookAccountExtension>();

    #[inline(always)]
    pub fn transferring(&self) -> bool {
        self.transferring != 0
    }

    #[inline(always)]
    pub fn set_transferring(&mut self, transferring: bool) {
        self.transferring = transferring as u8;
    }
}

// SAFETY: `TransferHookAccountExtension` is repr(C), contains only `u8`,
// has no padding, and all bit patterns are valid.
impl sealed::SealedPod for TransferHookAccountExtension {}
unsafe impl Pod for TransferHookAccountExtension {}

impl ExtensionValue for TransferHookAccountExtension {
    const TYPE: ExtensionType = ExtensionType::TransferHookAccount;
}
