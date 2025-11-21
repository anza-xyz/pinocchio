use pinocchio::pubkey::Pubkey;

/// Authorized data.
#[repr(C)]
pub struct Authorized {
    /// The pubkey authorized to stake.
    staker: Pubkey,

    /// The pubkey authorized to withdraw.
    withdrawer: Pubkey,
}

impl Authorized {
    /// The length of the `Authorized` data.
    pub const LEN: usize = core::mem::size_of::<Authorized>();

    /// Returns a reference to the staker pubkey.
    #[inline(always)]
    pub fn staker(&self) -> &Pubkey {
        &self.staker
    }

    /// Returns a reference to the withdrawer pubkey.
    #[inline(always)]
    pub fn withdrawer(&self) -> &Pubkey {
        &self.withdrawer
    }

    /// Returns the byte representation of the authorized data.
    #[inline(always)]
    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];
        bytes[..32].copy_from_slice(self.staker.as_ref());
        bytes[32..].copy_from_slice(self.withdrawer.as_ref());
        bytes
    }
}
