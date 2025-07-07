#![no_std]

use core::mem::MaybeUninit;

use pinocchio::pubkey::{Pubkey, MAX_SEEDS, PDA_MARKER};
#[cfg(target_os = "solana")]
use pinocchio::syscalls::sol_sha256;

#[cfg(feature = "const")]
use const_crypto::{bs58::decode_pubkey, sha2::Sha256};
#[cfg(feature = "const")]
pub use pinocchio::pubkey::Pubkey as __Pubkey;

/// Derive a [program address][pda] from the given seeds, bump and program id.
///
/// [pda]: https://solana.com/docs/core/pda
///
/// This function differs from [`pinocchio::pubkey::create_program_address`] in that it
/// does not perform a validation to ensure that the derived address is a valid
/// (off-curve) program derived address. It is intended for use in cases where the
/// seeds, bump, and program id are known to be valid, and the caller wants to derive
/// the address without incurring the cost of the `create_program_address` syscall.
pub fn derive_address<const N: usize>(seeds: &[&[u8]; N], bump: u8, program_id: &Pubkey) -> Pubkey {
    const {
        if N > MAX_SEEDS {
            panic!("number of seeds must be less than `MAX_SEEDS`");
        }
    }

    const UNINIT: MaybeUninit<&[u8]> = MaybeUninit::<&[u8]>::uninit();
    let mut data = [UNINIT; MAX_SEEDS + 3];
    let mut i = 0;

    while i < seeds.len() {
        // SAFETY: `data` is guanranteed to have enough space for `N` seeds,
        // so `i` will always be within bounds.
        unsafe {
            data.get_unchecked_mut(i).write(seeds.get_unchecked(i));
        }
        i += 1;
    }

    let bump = [bump];

    // SAFETY: `data` is guaranteed to have enough space for `MAX_SEEDS + 3`
    // elements, and `MAX_SEEDS` is as large as `N`.
    unsafe {
        data.get_unchecked_mut(i).write(bump.as_ref());
        data.get_unchecked_mut(i + 1).write(program_id.as_ref());
        data.get_unchecked_mut(i + 2).write(PDA_MARKER.as_ref());
    }

    #[cfg(target_os = "solana")]
    {
        let mut pda = MaybeUninit::<[u8; 32]>::uninit();

        // SAFETY: `data` has `i + 3` elements initialized.
        unsafe {
            sol_sha256(
                data.as_ptr() as *const _ as *const u8,
                (i + 3) as u64,
                pda.as_mut_ptr() as *mut _ as *mut u8,
            );
        }

        // SAFATE: `pda` has been initialized by the syscall.
        unsafe { pda.assume_init() }
    }

    #[cfg(not(target_os = "solana"))]
    unreachable!("deriving a pda is only available on target `solana`");
}

/// Derive a [program address][pda] from the given seeds, bump and program id.
///
/// [pda]: https://solana.com/docs/core/pda
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
    bump: u8,
    program_id: &__Pubkey,
) -> __Pubkey {
    const {
        if N > MAX_SEEDS {
            panic!("number of seeds must be less than `MAX_SEEDS`");
        }
    }

    let mut hasher = Sha256::new();
    let mut i = 0;

    while i < seeds.len() {
        hasher = hasher.update(seeds[i]);
        i += 1;
    }

    let bump = [bump];

    hasher
        .update(&bump)
        .update(program_id)
        .update(PDA_MARKER)
        .finalize()
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
        #[doc = "The const program ID."]
        pub const ID: $crate::__Pubkey = $crate::from_str($id);

        #[doc = "Returns `true` if given pubkey is the program ID."]
        #[inline]
        pub fn check_id(id: &$crate::__Pubkey) -> bool {
            id == &ID
        }

        #[doc = "Returns the program ID."]
        #[inline]
        pub const fn id() -> $crate::__Pubkey {
            ID
        }
    };
}

/// Create a `Pubkey` from a `&str`.
#[cfg(feature = "const")]
#[inline(always)]
pub const fn from_str(value: &str) -> __Pubkey {
    decode_pubkey(value)
}
