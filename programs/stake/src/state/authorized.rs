use core::mem::MaybeUninit;
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

    /// Writes the byte representation of the authorized data to the given slice.
    #[inline(always)]
    pub fn write_bytes(&self, dest: &mut [MaybeUninit<u8>]) {
        assert_eq!(dest.len(), Self::LEN);

        crate::write_bytes(&mut dest[..32], self.staker.as_ref());
        crate::write_bytes(&mut dest[32..], self.withdrawer.as_ref());
    }

    /// Returns the byte representation of the authorized data.
    #[inline(always)]
    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = core::mem::MaybeUninit::<[u8; Self::LEN]>::uninit();
        // SAFETY: We're writing to all Self::LEN bytes before reading.
        unsafe {
            let ptr = bytes.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(self.staker.as_ref().as_ptr(), ptr, 32);
            core::ptr::copy_nonoverlapping(self.withdrawer.as_ref().as_ptr(), ptr.add(32), 32);
            bytes.assume_init()
        }
    }
}
