#![no_std]

#[cfg(feature = "const")]
#[doc(hidden)]
// Re-export dependencies used in macros.
pub mod reexport {
    pub use pinocchio::Address;
}

use core::mem::MaybeUninit;
#[cfg(feature = "const")]
pub use five8_const::decode_32_const;
use pinocchio::address::{Address, MAX_SEEDS, PDA_MARKER};
#[cfg(target_os = "solana")]
use pinocchio::syscalls::sol_sha256;
#[cfg(feature = "const")]
use sha2_const_stable::Sha256;

/// Derive a [program address][pda] from the given seeds, optional bump and
/// program id.
///
/// [pda]: https://solana.com/docs/core/pda
///
/// In general, the derivation uses an optional bump (byte) value to ensure a
/// valid PDA (off-curve) is generated. Even when a program stores a bump to
/// derive a program address, it is necessary to use the
/// `Address::create_program_address` to validate the derivation. In
/// most cases, the program has the correct seeds for the derivation, so it would
/// be sufficient to just perform the derivation and compare it against the
/// expected resulting address.
///
/// This function avoids the cost of the `create_program_address` syscall
/// (`1500` compute units) by directly computing the derived address
/// calculating the hash of the seeds, bump and program id using the
/// `sol_sha256` syscall.
///
/// # Important
///
/// This function differs from `Address::create_program_address` in that
/// it does not perform a validation to ensure that the derived address is a valid
/// (off-curve) program derived address. It is intended for use in cases where the
/// seeds, bump, and program id are known to be valid, and the caller wants to derive
/// the address without incurring the cost of the `create_program_address` syscall.
pub fn derive_address<const N: usize>(
    seeds: &[&[u8]; N],
    bump: Option<u8>,
    program_id: &Address,
) -> Address {
    const {
        assert!(N < MAX_SEEDS, "number of seeds must be less than MAX_SEEDS");
    }

    const UNINIT: MaybeUninit<&[u8]> = MaybeUninit::<&[u8]>::uninit();
    let mut data = [UNINIT; MAX_SEEDS + 2];
    let mut i = 0;

    while i < N {
        // SAFETY: `data` is guaranteed to have enough space for `N` seeds,
        // so `i` will always be within bounds.
        unsafe {
            data.get_unchecked_mut(i).write(seeds.get_unchecked(i));
        }
        i += 1;
    }

    // TODO: replace this with `as_slice` when the MSRV is upgraded
    // to `1.84.0+`.
    let bump_seed = [bump.unwrap_or_default()];

    // SAFETY: `data` is guaranteed to have enough space for `MAX_SEEDS + 2`
    // elements, and `MAX_SEEDS` is as large as `N`.
    unsafe {
        if bump.is_some() {
            data.get_unchecked_mut(i).write(&bump_seed);
            i += 1;
        }
        data.get_unchecked_mut(i).write(program_id.as_ref());
        data.get_unchecked_mut(i + 1).write(PDA_MARKER.as_ref());
    }

    #[cfg(target_os = "solana")]
    {
        let mut pda = MaybeUninit::<Address>::uninit();

        // SAFETY: `data` has `i + 2` elements initialized.
        unsafe {
            sol_sha256(
                data.as_ptr() as *const u8,
                (i + 2) as u64,
                pda.as_mut_ptr() as *mut u8,
            );
        }

        // SAFETY: `pda` has been initialized by the syscall.
        unsafe { pda.assume_init() }
    }

    #[cfg(not(target_os = "solana"))]
    unreachable!("deriving a pda is only available on target `solana`");
}

/// Derive a [program address][pda] from the given seeds, optional bump and
/// program id.
///
/// [pda]: https://solana.com/docs/core/pda
///
/// In general, the derivation uses an optional bump (byte) value to ensure a
/// valid PDA (off-curve) is generated.
///
/// This function is intended for use in `const` contexts - i.e., the seeds and
/// bump are known at compile time and the program id is also a constant. It avoids
/// the cost of the `create_program_address` syscall (`1500` compute units) by
/// directly computing the derived address using the SHA-256 hash of the seeds,
/// bump and program id.
///
/// # Important
///
/// This function differs from `Address::create_program_address` in that
/// it does not perform a validation to ensure that the derived address is a valid
/// (off-curve) program derived address. It is intended for use in cases where the
/// seeds, bump, and program id are known to be valid, and the caller wants to derive
/// the address without incurring the cost of the `create_program_address` syscall.
///
/// This function is a compile-time constant version of [`derive_address`].
#[cfg(feature = "const")]
pub const fn derive_address_const<const N: usize>(
    seeds: &[&[u8]; N],
    bump: Option<u8>,
    program_id: &Address,
) -> Address {
    use pinocchio::address::PDA_MARKER;

    const {
        assert!(N < MAX_SEEDS, "number of seeds must be less than MAX_SEEDS");
    }

    let mut hasher = Sha256::new();
    let mut i = 0;

    while i < seeds.len() {
        hasher = hasher.update(seeds[i]);
        i += 1;
    }

    Address::new_from_array(
        hasher
            .update(bump.as_slice())
            .update(program_id.as_array())
            .update(PDA_MARKER)
            .finalize(),
    )
}

/// Convenience macro to define a static [`Address`] value representing the program ID.
///
/// This macro also defines a helper function to check whether a given address is
/// equal to the program ID.
#[cfg(feature = "const")]
#[macro_export]
macro_rules! declare_id {
    ( $id:expr ) => {
        #[doc = "The constant program ID."]
        pub const ID: $crate::reexport::Address = $crate::from_str($id);

        #[doc = "Returns `true` if the given address is equal to the program ID."]
        #[inline]
        pub fn check_id(id: &$crate::reexport::Address) -> bool {
            id == &ID
        }

        #[doc = "Returns the program ID."]
        #[inline]
        pub const fn id() -> $crate::reexport::Address {
            ID
        }
    };
}

/// Create an `Address` from a `&str`.
#[cfg(feature = "const")]
#[inline(always)]
pub const fn from_str(value: &str) -> Address {
    Address::new_from_array(decode_32_const(value))
}
