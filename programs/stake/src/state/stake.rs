use crate::state::Delegation;

/// Stake data.
#[repr(C)]
pub struct Stake {
    /// The delegation information.
    delegation: Delegation,

    /// Credits observed for calculating rewards.
    credits_observed: [u8; 8],
}

impl Stake {
    /// The length of the `Stake` data.
    pub const LEN: usize = core::mem::size_of::<Stake>();

    /// Returns a reference to the delegation information.
    #[inline(always)]
    pub fn delegation(&self) -> &Delegation {
        &self.delegation
    }

    /// Returns the credits observed.
    #[inline(always)]
    pub fn credits_observed(&self) -> u64 {
        u64::from_le_bytes(self.credits_observed)
    }
}