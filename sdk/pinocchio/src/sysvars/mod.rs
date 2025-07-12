//! Provides access to cluster system accounts.

#[cfg(target_os = "solana")]
use crate::syscalls::sol_get_sysvar;
use crate::{program_error::ProgramError, pubkey::Pubkey};
#[cfg(not(target_os = "solana"))]
use core::hint::black_box;

pub mod clock;
pub mod fees;
pub mod instructions;
pub mod rent;
pub mod slot_hashes;

/// Return value indicating that the `offset + length` is greater than the length of
/// the sysvar data.
//
// Defined in the bpf loader as [`OFFSET_LENGTH_EXCEEDS_SYSVAR`](https://github.com/anza-xyz/agave/blob/master/programs/bpf_loader/src/syscalls/sysvar.rs#L172).
#[cfg(target_os = "solana")]
const OFFSET_LENGTH_EXCEEDS_SYSVAR: u64 = 1;

/// Return value indicating that the sysvar was not found.
//
// Defined in the bpf loader as [`SYSVAR_NOT_FOUND`](https://github.com/anza-xyz/agave/blob/master/programs/bpf_loader/src/syscalls/sysvar.rs#L171).
#[cfg(target_os = "solana")]
const SYSVAR_NOT_FOUND: u64 = 2;

/// A type that holds sysvar data.
pub trait Sysvar: Sized {
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
                $crate::SUCCESS => {
                    // SAFETY: The syscall initialized the memory.
                    Ok(unsafe { var.assume_init() })
                }
                // Unexpected errors are folded into `UnsupportedSysvar`.
                _ => Err($crate::program_error::ProgramError::UnsupportedSysvar),
            }
        }
    };
}

/// Handler for retrieving a slice of sysvar data from the `sol_get_sysvar`
/// syscall.
///
/// # Safety
///
/// The caller must ensure that the `dst` pointer is valid and has enough space
/// to hold the requested `len` bytes of data.
#[inline]
pub unsafe fn get_sysvar_unchecked(
    dst: *mut u8,
    sysvar_id: &Pubkey,
    offset: usize,
    len: usize,
) -> Result<(), ProgramError> {
    #[cfg(target_os = "solana")]
    {
        let result = unsafe {
            sol_get_sysvar(
                sysvar_id as *const _ as *const u8,
                dst,
                offset as u64,
                len as u64,
            )
        };

        match result {
            crate::SUCCESS => Ok(()),
            OFFSET_LENGTH_EXCEEDS_SYSVAR => Err(ProgramError::InvalidArgument),
            SYSVAR_NOT_FOUND => Err(ProgramError::UnsupportedSysvar),
            // Unexpected errors are folded into `UnsupportedSysvar`.
            _ => Err(ProgramError::UnsupportedSysvar),
        }
    }

    #[cfg(not(target_os = "solana"))]
    {
        black_box((dst, sysvar_id, offset, len));
        Ok(())
    }
}

/// Handler for retrieving a slice of sysvar data from the `sol_get_sysvar`
/// syscall.
#[inline(always)]
pub fn get_sysvar(dst: &mut [u8], sysvar_id: &Pubkey, offset: usize) -> Result<(), ProgramError> {
    // SAFETY: Use the length of the slice as the length parameter.
    unsafe { get_sysvar_unchecked(dst.as_mut_ptr(), sysvar_id, offset, dst.len()) }
}
