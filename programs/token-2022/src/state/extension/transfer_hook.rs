use {
    super::{sealed, ExtensionType, ExtensionValue, Pod},
    solana_address::Address,
};

/// Transfer hook extension data for mints (64 bytes).
///
/// Configures a custom program to execute additional logic on every
/// transfer involving this mint.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TransferHookExtension {
    authority: [u8; 32],
    program_id: [u8; 32],
}

impl TransferHookExtension {
    pub const LEN: usize = core::mem::size_of::<TransferHookExtension>();

    #[inline(always)]
    pub fn authority(&self) -> &Address {
        // SAFETY: `Address` is `#[repr(transparent)]` over `[u8; 32]` with
        // alignment 1, so the pointer cast is valid.
        unsafe { &*(self.authority.as_ptr() as *const Address) }
    }

    #[inline(always)]
    pub fn program_id(&self) -> &Address {
        // SAFETY: `Address` is `#[repr(transparent)]` over `[u8; 32]` with
        // alignment 1, so the pointer cast is valid.
        unsafe { &*(self.program_id.as_ptr() as *const Address) }
    }

    #[inline(always)]
    pub fn set_authority(&mut self, authority: &Address) {
        self.authority.copy_from_slice(authority.as_ref());
    }

    #[inline(always)]
    pub fn set_program_id(&mut self, program_id: &Address) {
        self.program_id.copy_from_slice(program_id.as_ref());
    }
}

// SAFETY: `TransferHookExtension` is repr(C), contains only `[u8; 32]` arrays,
// has no padding, and all bit patterns are valid.
impl sealed::SealedPod for TransferHookExtension {}
unsafe impl Pod for TransferHookExtension {}

impl ExtensionValue for TransferHookExtension {
    const TYPE: ExtensionType = ExtensionType::TransferHook;
}
