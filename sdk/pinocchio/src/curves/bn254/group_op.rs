//! Group operations on the BN254 curve.

use super::{ALT_BN128_FIELD_SIZE, ALT_BN128_G1_POINT_SIZE, ALT_BN128_G2_POINT_SIZE};
use crate::program_error::ProgramError;

#[cfg(target_os = "solana")]
use crate::syscalls::sol_alt_bn128_group_op;

/// Input length for the add operation.
pub const ALT_BN128_ADDITION_INPUT_LEN: usize = ALT_BN128_G1_POINT_SIZE * 2; // 128

/// Input length for the multiplication operation.
pub const ALT_BN128_MULTIPLICATION_INPUT_LEN: usize =
    ALT_BN128_G1_POINT_SIZE + ALT_BN128_FIELD_SIZE; // 96

/// Pair element length.
pub const ALT_BN128_PAIRING_ELEMENT_LEN: usize = ALT_BN128_G1_POINT_SIZE + ALT_BN128_G2_POINT_SIZE; // 192

/// Output length for the add operation.
pub const ALT_BN128_ADDITION_OUTPUT_LEN: usize = ALT_BN128_G1_POINT_SIZE; // 64

/// Output length for the multiplication operation.
pub const ALT_BN128_MULTIPLICATION_OUTPUT_LEN: usize = ALT_BN128_G1_POINT_SIZE; // 64

const ALT_BN128_ADD: u64 = 0;
#[allow(dead_code)]
const ALT_BN128_SUB: u64 = 1; // not implemented in the syscall
const ALT_BN128_MUL: u64 = 2;
const ALT_BN128_PAIRING: u64 = 3;

/// Add two G1 points on the BN254 curve in big-endian (EIP-197) encoding.
///
/// # Arguments
///
/// * `input` - Two consecutive G1 points in big-endian (EIP-197) encoding.
///
/// # Returns
///
/// A `Result` containing the result of the addition in big-endian (EIP-197) encoding,
/// or an error if the input is invalid.
///
/// Note: This function does **not** check if the input has the correct length.
/// It will return an error if the length is invalid, incurring the cost of the syscall.
#[inline(always)]
pub fn alt_bn128_addition(
    input: &[u8],
) -> Result<[u8; ALT_BN128_ADDITION_OUTPUT_LEN], ProgramError> {
    alt_bn128_group_op(input, ALT_BN128_ADD)
}

/// Multiply a G1 point by a scalar on the BN254 curve in big-endian (EIP-197) encoding.
///
/// # Arguments
///
/// * `input` - A G1 point in big-endian (EIP-197) encoding,
///   followed by a scalar in big-endian (EIP-197) encoding.
///
/// # Returns
///
/// A `Result` containing the result of the multiplication in big-endian (EIP-197)
/// encoding, or an error if the input is invalid.
///
/// Note: This function does **not** check if the input has the correct length.
/// It will return an error if the length is invalid, incurring the cost of the syscall.
#[inline(always)]
pub fn alt_bn128_multiplication(
    input: &[u8],
) -> Result<[u8; ALT_BN128_MULTIPLICATION_OUTPUT_LEN], ProgramError> {
    alt_bn128_group_op(input, ALT_BN128_MUL)
}

/// Perform a pairing operation on the BN254 curve in big-endian (EIP-197) encoding.
///
/// # Arguments
///
/// * `input` - A sequence of pairs of G1 and G2 points in big-endian (EIP-197) encoding.
///
/// # Returns
///
/// A `Result` containing the result of the pairing operation, or an error if the input is invalid.
///
/// Note: This function does **not** check if the input has the correct length.
/// Currently, if the length is invalid, it will not return an error; instead it will use only
/// multiples of [`ALT_BN128_PAIRING_ELEMENT_LEN`] bytes and discard the rest.
/// After SIMD-0334 is implemented, it will return an error if the length is invalid,
/// incurring the cost of the syscall.
#[inline(always)]
pub fn alt_bn128_pairing(input: &[u8]) -> Result<u8, ProgramError> {
    alt_bn128_group_op::<32>(input, ALT_BN128_PAIRING).map(|data| data[31])
}

#[inline]
fn alt_bn128_group_op<const OUTPUT_DATA_LEN: usize>(
    input: &[u8],
    op: u64,
) -> Result<[u8; OUTPUT_DATA_LEN], ProgramError> {
    // Call via a system call to perform the calculation
    #[cfg(target_os = "solana")]
    {
        let mut bytes = core::mem::MaybeUninit::<[u8; OUTPUT_DATA_LEN]>::uninit();

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
