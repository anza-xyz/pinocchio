use pinocchio::pubkey::Pubkey;

/// Lockup data.
#[repr(C)]
pub struct Lockup {
    /// Unix timestamp at which the lockup expires.
    unix_timestamp: [u8; 8],
    /// Epoch at which the lockup expires.
    epoch: [u8; 8],
    /// The custodian pubkey that can modify the lockup.
    custodian: Pubkey,
}

impl Lockup {
    /// The length of the `Lockup` data.
    pub const LEN: usize = core::mem::size_of::<Lockup>();

    /// Returns the unix timestamp at which the lockup expires.
    #[inline(always)]
    pub fn unix_timestamp(&self) -> i64 {
        i64::from_le_bytes(self.unix_timestamp)
    }

    /// Returns the epoch at which the lockup expires.
    #[inline(always)]
    pub fn epoch(&self) -> u64 {
        u64::from_le_bytes(self.epoch)
    }

    /// Returns a reference to the custodian pubkey.
    #[inline(always)]
    pub fn custodian(&self) -> &Pubkey {
        &self.custodian
    }

    /// Returns the byte representation of the lockup data.
    #[inline(always)]
    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0u8; Self::LEN];
        bytes[..8].copy_from_slice(&self.unix_timestamp);
        bytes[8..16].copy_from_slice(&self.epoch);
        bytes[16..].copy_from_slice(self.custodian.as_ref());
        bytes
    }
}
