/// StakeFlags data.
#[repr(C)]
pub struct StakeFlags {
    /// The stake flags bits.
    bits: u8,
}

impl StakeFlags {
    /// The length of the `StakeFlags` data.
    pub const LEN: usize = core::mem::size_of::<StakeFlags>();

    /// Must withdraw to account must be fully de-activated before withdraw.
    pub const MUST_FULLY_ACTIVATE_BEFORE_DEACTIVATION_IS_PERMITTED: u8 = 0b0000_0001;

    /// Returns the raw bits.
    #[inline(always)]
    pub fn bits(&self) -> u8 {
        self.bits
    }

    /// Returns whether the stake must be fully activated before deactivation is permitted.
    #[inline(always)]
    pub fn must_fully_activate_before_deactivation_is_permitted(&self) -> bool {
        self.bits & Self::MUST_FULLY_ACTIVATE_BEFORE_DEACTIVATION_IS_PERMITTED != 0
    }
}
