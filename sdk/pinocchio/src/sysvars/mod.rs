//! Provides access to cluster system accounts.

use crate::program_error::ProgramError;

pub mod clock;
pub mod fees;
pub mod rent;

// JC: the `Sysvar` trait in `solana-sysvar` is very heavy and adds too
// much complexity, so this is a great addition.
//
// We could keep this new trait, add it to a new crate, but have the trait use
// the generic `get_sysvar` syscall instead of the sysvar-specific ones
// (ie.`sol_get_rent_sysvar`). And then we could have each of `solana-clock`,
// `solana-rent`, etc implement this lower-level trait in a nostd compatible
// way, for pinocchio to re-use, if some feature on the `solana-clock` crate
// enabled. What do you think?

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
