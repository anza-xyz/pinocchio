use {
    super::{sealed, ExtensionPod, ExtensionType, ExtensionValue},
    solana_address::Address,
};

/// Permanent delegate extension data for mints (32 bytes).
///
/// When set on a mint, the delegate has unrestricted transfer and
/// burn authority over all token accounts for the mint.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PermanentDelegateExtension {
    delegate: Address,
}

impl PermanentDelegateExtension {
    pub const LEN: usize = core::mem::size_of::<PermanentDelegateExtension>();

    #[inline(always)]
    pub fn delegate(&self) -> &Address {
        &self.delegate
    }

    #[inline(always)]
    pub fn set_delegate(&mut self, delegate: &Address) {
        self.delegate = delegate.clone();
    }
}

// SAFETY: `PermanentDelegateExtension` is repr(C), contains only `Address`
// (`[u8; 32]`), has no padding, and all bit patterns are valid.
impl sealed::SealedExtensionPod for PermanentDelegateExtension {}
unsafe impl ExtensionPod for PermanentDelegateExtension {}

impl ExtensionValue for PermanentDelegateExtension {
    const TYPE: ExtensionType = ExtensionType::PermanentDelegate;
}
