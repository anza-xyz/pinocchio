use core::mem::MaybeUninit;
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

    /// Writes the byte representation of the lockup data to the given slice.
    #[inline(always)]
    pub fn write_bytes(&self, dest: &mut [MaybeUninit<u8>]) {
        assert_eq!(dest.len(), Self::LEN);

        crate::write_bytes(&mut dest[..8], &self.unix_timestamp);
        crate::write_bytes(&mut dest[8..16], &self.epoch);
        crate::write_bytes(&mut dest[16..], self.custodian.as_ref());
    }

    /// Returns the byte representation of the lockup data.
    #[inline(always)]
    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = core::mem::MaybeUninit::<[u8; Self::LEN]>::uninit();
        // SAFETY: We're writing to all Self::LEN bytes before reading.
        unsafe {
            let ptr = bytes.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(self.unix_timestamp.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(self.epoch.as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(self.custodian.as_ref().as_ptr(), ptr.add(16), 32);
            bytes.assume_init()
        }
    }
}
