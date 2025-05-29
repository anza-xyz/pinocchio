//! Provides access to cluster system accounts.

use crate::{program_error::ProgramError, pubkey::Pubkey};

pub mod clock;
pub mod fees;
pub mod instructions;
pub mod rent;

/// A type that holds sysvar data.
pub trait Sysvar: Default + Sized {
    /// Load the sysvar directly from the runtime.
    ///
    /// This is the preferred way to load a sysvar. Calling this method does not
    /// incur any deserialization overhead, and does not require the sysvar
    /// account to be passed to the program.
    ///
    /// Not all sysvars support this method. If not, it returns
    /// [`ProgramError::UnsupportedSysvar`].
    fn get() -> Result<Self, ProgramError> {
        Err(ProgramError::UnsupportedSysvar)
    }
}

/// Implements the [`Sysvar::get`] method for both SBF and host targets.
#[macro_export]
macro_rules! impl_sysvar_get {
    ($syscall_name:ident) => {
        fn get() -> Result<Self, $crate::program_error::ProgramError> {
            let mut var = core::mem::MaybeUninit::<Self>::uninit();
            let var_addr = var.as_mut_ptr() as *mut _ as *mut u8;

            #[cfg(target_os = "solana")]
            let result = unsafe { $crate::syscalls::$syscall_name(var_addr) };

            #[cfg(not(target_os = "solana"))]
            let result = core::hint::black_box(var_addr as *const _ as u64);

            match result {
                // SAFETY: The syscall initialized the memory.
                $crate::SUCCESS => Ok(unsafe { var.assume_init() }),
                e => Err(e.into()),
            }
        }
    };
}

/// Handler for retrieving a slice of sysvar data from the `sol_get_sysvar`
/// syscall.
///
/// # Safety
///
/// `dst` must point to a buffer that can hold at least `length` bytes.
#[inline]
pub unsafe fn get_sysvar_unchecked(
    dst: *mut u8,
    sysvar_id: &Pubkey,
    offset: u64,
    length: u64,
) -> Result<(), ProgramError> {
    let sysvar_id = sysvar_id as *const _ as *const u8;

    #[cfg(target_os = "solana")]
    let result = unsafe { crate::syscalls::sol_get_sysvar(sysvar_id, dst, offset, length) };

    #[cfg(not(target_os = "solana"))]
    let result = core::hint::black_box(sysvar_id as u64 + dst as u64 + offset + length);

    match result {
        crate::SUCCESS => Ok(()),
        e => Err(e.into()),
    }
}

/// Handler for retrieving a slice of sysvar data from the `sol_get_sysvar`
/// syscall.
#[inline]
pub fn get_sysvar(
    dst: &mut [u8],
    sysvar_id: &Pubkey,
    offset: u64,
    length: u64,
) -> Result<(), ProgramError> {
    // Check that the provided destination buffer is large enough to hold the
    // requested data.
    if dst.len() < length as usize {
        return Err(ProgramError::InvalidArgument);
    }

    unsafe { get_sysvar_unchecked(dst as *mut _ as *mut u8, sysvar_id, offset, length) }
}
