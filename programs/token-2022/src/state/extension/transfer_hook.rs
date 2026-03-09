use {
    super::{sealed, ExtensionPod, ExtensionType, ExtensionValue},
    solana_address::Address,
};

/// Transfer hook extension data for mints (64 bytes).
///
/// Configures a custom program to execute additional logic on every
/// transfer involving this mint.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransferHookExtension {
    authority: Address,
    program_id: Address,
}

impl TransferHookExtension {
    pub const LEN: usize = core::mem::size_of::<TransferHookExtension>();

    #[inline(always)]
    pub fn authority(&self) -> &Address {
        &self.authority
    }

    #[inline(always)]
    pub fn program_id(&self) -> &Address {
        &self.program_id
    }

    #[inline(always)]
    pub fn set_authority(&mut self, authority: &Address) {
        self.authority = authority.clone();
    }

    #[inline(always)]
    pub fn set_program_id(&mut self, program_id: &Address) {
        self.program_id = program_id.clone();
    }
}

// SAFETY: `TransferHookExtension` is repr(C), contains only `Address`
// (`[u8; 32]`) fields, has no padding, and all bit patterns are valid.
impl sealed::SealedExtensionPod for TransferHookExtension {}
unsafe impl ExtensionPod for TransferHookExtension {}

impl ExtensionValue for TransferHookExtension {
    const TYPE: ExtensionType = ExtensionType::TransferHook;
}
