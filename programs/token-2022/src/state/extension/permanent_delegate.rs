use {
    super::{sealed, ExtensionType, ExtensionValue, Pod},
    solana_address::Address,
};

/// Permanent delegate extension data for mints (32 bytes).
///
/// When set on a mint, the delegate has unrestricted transfer and
/// burn authority over all token accounts for the mint.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PermanentDelegateExtension {
    delegate: [u8; 32],
}

impl PermanentDelegateExtension {
    pub const LEN: usize = core::mem::size_of::<PermanentDelegateExtension>();

    #[inline(always)]
    pub fn delegate(&self) -> &Address {
        // SAFETY: `Address` is `#[repr(transparent)]` over `[u8; 32]` with
        // alignment 1, so the pointer cast is valid.
        unsafe { &*(self.delegate.as_ptr() as *const Address) }
    }

    #[inline(always)]
    pub fn set_delegate(&mut self, delegate: &Address) {
        self.delegate.copy_from_slice(delegate.as_ref());
    }
}

// SAFETY: `PermanentDelegateExtension` is repr(C), contains only `[u8; 32]`,
// has no padding, and all bit patterns are valid.
impl sealed::SealedPod for PermanentDelegateExtension {}
unsafe impl Pod for PermanentDelegateExtension {}

impl ExtensionValue for PermanentDelegateExtension {
    const TYPE: ExtensionType = ExtensionType::PermanentDelegate;
}
