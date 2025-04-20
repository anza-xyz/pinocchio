//! Provides access to cluster system accounts.

use crate::program_error::ProgramError;

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
            let mut var = Self::default();
            let var_addr = &mut var as *mut _ as *mut u8;

            #[cfg(target_os = "solana")]
            let result = unsafe { $crate::syscalls::$syscall_name(var_addr) };

            #[cfg(not(target_os = "solana"))]
            let result = core::hint::black_box(var_addr as *const _ as u64);

            match result {
                $crate::SUCCESS => Ok(var),
                e => Err(e.into()),
            }
        }
    };
}
