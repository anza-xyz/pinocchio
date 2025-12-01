use crate::state::{Authorized, Lockup};

/// Meta data.
#[repr(C)]
pub struct Meta {
    /// The amount of stake that must remain in the account to be rent exempt.
    rent_exempt_reserve: [u8; 8],
    /// The authorized staker and withdrawer.
    authorized: Authorized,
    /// Lockup information.
    lockup: Lockup,
}

impl Meta {
    /// The length of the `Meta` data.
    pub const LEN: usize = core::mem::size_of::<Meta>();

    /// Returns the rent exempt reserve.
    #[inline(always)]
    pub fn rent_exempt_reserve(&self) -> u64 {
        u64::from_le_bytes(self.rent_exempt_reserve)
    }

    /// Returns a reference to the authorized staker and withdrawer.
    #[inline(always)]
    pub fn authorized(&self) -> &Authorized {
        &self.authorized
    }

    /// Returns a reference to the lockup information.
    #[inline(always)]
    pub fn lockup(&self) -> &Lockup {
        &self.lockup
    }
}
