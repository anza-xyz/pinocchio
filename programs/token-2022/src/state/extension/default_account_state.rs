use {
    super::{sealed, ExtensionType, ExtensionValue, Pod},
    crate::state::AccountState,
    solana_program_error::ProgramError,
};

/// Default account state extension data (1 byte).
///
/// When set on a mint, all new token accounts are initialized
/// with this state.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DefaultAccountStateExtension {
    state: u8,
}

impl DefaultAccountStateExtension {
    pub const LEN: usize = core::mem::size_of::<DefaultAccountStateExtension>();

    #[inline(always)]
    pub fn state(&self) -> Result<AccountState, ProgramError> {
        AccountState::try_from(self.state)
    }

    #[inline(always)]
    pub fn set_state(&mut self, state: AccountState) {
        self.state = state as u8;
    }
}

// SAFETY: `DefaultAccountStateExtension` is repr(C), contains only `u8`,
// has no padding, and all bit patterns are valid.
impl sealed::SealedPod for DefaultAccountStateExtension {}
unsafe impl Pod for DefaultAccountStateExtension {}

impl ExtensionValue for DefaultAccountStateExtension {
    const TYPE: ExtensionType = ExtensionType::DefaultAccountState;
}
