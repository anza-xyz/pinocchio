//! BN254 curve operations

pub mod addition;
pub mod compression;
pub mod multiplication;
pub mod pairing;

pub use addition::*;
pub use compression::*;
pub use multiplication::*;
pub use pairing::*;

use crate::program_error::ProgramError;

#[cfg(target_os = "solana")]
use crate::syscalls::sol_alt_bn128_group_op;

/// Size of the EC point field, in bytes.
pub const ALT_BN128_FIELD_SIZE: usize = 32;
/// A group element in G1 consists of two field elements `(x, y)`.
pub const ALT_BN128_G1_POINT_SIZE: usize = ALT_BN128_FIELD_SIZE * 2;
/// Elements in G2 is represented by 2 field-extension elements `(x, y)`.
pub const ALT_BN128_G2_POINT_SIZE: usize = ALT_BN128_FIELD_SIZE * 4;

#[inline]
fn alt_bn128_group_op<const OUTPUT_DATA_SIZE: usize>(
    input: &[u8],
    op: u64,
) -> Result<[u8; OUTPUT_DATA_SIZE], ProgramError> {
    // Call via a system call to perform the calculation
    #[cfg(target_os = "solana")]
    {
        let mut bytes = core::mem::MaybeUninit::<[u8; OUTPUT_DATA_SIZE]>::uninit();

        let result = unsafe {
            sol_alt_bn128_group_op(
                op,
                input as *const _ as *const u8,
                input.len() as u64,
                bytes.as_mut_ptr() as *mut u8,
            )
        };

        match result {
            // SAFETY: The syscall has initialized the bytes.
            crate::SUCCESS => Ok(unsafe { bytes.assume_init() }),
            _ => Err(ProgramError::InvalidArgument),
        }
    }

    #[cfg(not(target_os = "solana"))]
    {
        core::hint::black_box((input, op));
        panic!("alt_bn128_group_op is only available on target `solana`")
    }
}
