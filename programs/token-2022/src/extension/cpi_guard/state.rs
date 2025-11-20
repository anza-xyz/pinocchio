use crate::extension::consts::ExtensionDiscriminator;
use crate::{write_bytes, UNINIT_BYTE};
use core::mem::MaybeUninit;
use pinocchio::program_error::ProgramError;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CpiGuardInstruction {
    Enable,
    Disable,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CpiGuard {
    /// Lock privileged token operations from happening via CPI
    pub lock_cpi: u8,
}

impl CpiGuard {
    /// The length of the account with `CpiGuard` extension data
    const LEN: u8 = 171;
    /// The index where CPI guard data starts in the account with `CpiGuard` extension data
    const CPI_GUARD_START: u8 = 170;

    /// The length of the `CpiGuard` extension data.
    pub const BASE_LEN: usize = core::mem::size_of::<CpiGuard>();

    /// Return a `CpiGuard` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. `bytes` contains at least `LEN` bytes
    /// 2. `bytes` contains a valid representation of `CpiGuard`
    /// 3. The data is properly aligned
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes[Self::CPI_GUARD_START as usize..].as_ptr() as *const CpiGuard)
    }

    /// Safe version of from_bytes that performs validation
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() < Self::LEN as usize {
            Err(ProgramError::InvalidAccountData)?;
        }

        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    #[inline(always)]
    pub fn lock_cpi(&self) -> bool {
        self.lock_cpi != 0
    }
}

pub fn cpi_guard_instruction_data(instruction_type: CpiGuardInstruction) -> [MaybeUninit<u8>; 2] {
    // instruction data
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    let mut data = [UNINIT_BYTE; 2];
    // Set extension discriminator at offset [0]
    write_bytes(&mut data, &[ExtensionDiscriminator::CpiGuard as u8]);
    // Set sub-instruction at offset [1]
    write_bytes(&mut data[1..2], &[instruction_type as u8]);

    data
}
