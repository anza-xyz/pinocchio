#![no_std]

#[cfg(feature = "const")]
#[doc(hidden)]
// Re-export dependencies used in macros.
pub mod reexport {
    pub use pinocchio::pubkey::Pubkey;
}

#[cfg(feature = "const")]
use const_crypto::{bs58::decode_pubkey, sha2::Sha256};
use core::mem::MaybeUninit;
use pinocchio::pubkey::{Pubkey, MAX_SEEDS, PDA_MARKER};
#[cfg(target_os = "solana")]
use pinocchio::syscalls::sol_sha256;

/// Derive a [program address][pda] from the given its seeds and program id.
///
/// [pda]: https://solana.com/docs/core/pda
///
/// In general, the seeds include a bump (byte) value to ensure a valid PDA
/// (off-curve) is generated. The bump is typically the last byte of the seeds
/// array. Even when a program stores a bump to derive a program address, it is
/// necessary to use the [`pinocchio::pubkey::create_program_address`] to validate
/// the derivation. In most cases, the program has the correct seeds for the
/// derivation, so it would be sufficient to just perform the derivation and compare
/// it against the expected resulting address.
///
/// This function avoids the cost of the `create_program_address` syscall
/// (`1500` compute units) by directly computing the derived address
/// calculating the hash of the seeds and program id using the
/// `sol_sha256` syscall.
///
/// # Important
///
/// This function differs from [`pinocchio::pubkey::create_program_address`] in that it
/// does not perform a validation to ensure that the derived address is a valid
/// (off-curve) program derived address. It is intended for use in cases where the
/// seeds and program id are known to be valid, and the caller wants to derive
/// the address without incurring the cost of the `create_program_address` syscall.
pub fn derive_address<const N: usize>(seeds: &[&[u8]; N], program_id: &Pubkey) -> Pubkey {
    const {
        assert!(
            N <= MAX_SEEDS,
            "number of seeds must be less than MAX_SEEDS"
        );
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

    // SAFETY: `data` is guaranteed to have enough space for `MAX_SEEDS + 2`
    // elements, and `MAX_SEEDS` is as large as `N`.
    unsafe {
        data.get_unchecked_mut(i).write(program_id.as_ref());
        data.get_unchecked_mut(i + 1).write(PDA_MARKER.as_ref());
    }

    #[cfg(target_os = "solana")]
    {
        let mut pda = MaybeUninit::<[u8; 32]>::uninit();

        // SAFETY: `data` has `N + 2` elements initialized.
        unsafe {
            sol_sha256(
                data.as_ptr() as *const u8,
                (N + 2) as u64,
                pda.as_mut_ptr() as *mut u8,
            );
        }

        // SAFETY: `pda` has been initialized by the syscall.
        unsafe { pda.assume_init() }
    }

    #[cfg(not(target_os = "solana"))]
    unreachable!("deriving a pda is only available on target `solana`");
}

/// Derive a [program address][pda] from the given seeds and program id.
///
/// [pda]: https://solana.com/docs/core/pda
///
/// In general, the seeds include a bump (byte) value to ensure a valid PDA
/// (off-curve) is generated. The bump is typically the last byte of the seeds
/// array.
///
/// This function avoids the cost of the `create_program_address` syscall
/// (`1500` compute units) by directly computing the derived address
/// using the SHA-256 hash of the seeds and program id.
///
/// This function is intended for use in `const` contexts - i.e., the seeds are
/// known at compile time and the program id is also a constant.
///
/// # Important
///
/// This function differs from [`pinocchio::pubkey::create_program_address`] in that it
/// does not perform a validation to ensure that the derived address is a valid
/// (off-curve) program derived address. It is intended for use in cases where the
/// seeds, bump, and program id are known to be valid, and the caller wants to derive
/// the address without incurring the cost of the `create_program_address` syscall.
///
/// This function is a compile-time constant version of [`derive_address`].
#[cfg(feature = "const")]
pub const fn derive_address_const<const N: usize>(
    seeds: &[&[u8]; N],
    program_id: &Pubkey,
) -> Pubkey {
    const {
        assert!(
            N <= MAX_SEEDS,
            "number of seeds must be less than MAX_SEEDS"
        );
    }

    let mut hasher = Sha256::new();
    let mut i = 0;

    while i < seeds.len() {
        hasher = hasher.update(seeds[i]);
        i += 1;
    }

    hasher.update(program_id).update(PDA_MARKER).finalize()
}

/// Convenience macro to define a static `Pubkey` value.
#[cfg(feature = "const")]
#[macro_export]
macro_rules! pubkey {
    ( $id:literal ) => {
        $crate::from_str($id)
    };
}

/// Convenience macro to define a static `Pubkey` value representing the program ID.
///
/// This macro also defines a helper function to check whether a given pubkey is
/// equal to the program ID.
#[cfg(feature = "const")]
#[macro_export]
macro_rules! declare_id {
    ( $id:expr ) => {
        use $crate::reexport::Pubkey;

        #[doc = "The const program ID."]
        pub const ID: Pubkey = $crate::from_str($id);

        #[doc = "Returns `true` if given pubkey is the program ID."]
        #[inline]
        pub fn check_id(id: &Pubkey) -> bool {
            id == &ID
        }

        #[doc = "Returns the program ID."]
        #[inline]
        pub const fn id() -> Pubkey {
            ID
        }
    };
}

/// Create a `Pubkey` from a `&str`.
#[cfg(feature = "const")]
#[inline(always)]
pub const fn from_str(value: &str) -> Pubkey {
    decode_pubkey(value)
}
