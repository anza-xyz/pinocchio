//! Pairing operations on the BN254 curve.

use super::{alt_bn128_group_op, ALT_BN128_G1_POINT_SIZE, ALT_BN128_G2_POINT_SIZE};
use crate::program_error::ProgramError;

/// Pair element size.
pub const ALT_BN128_PAIRING_ELEMENT_SIZE: usize = ALT_BN128_G1_POINT_SIZE + ALT_BN128_G2_POINT_SIZE; // 192

const ALT_BN128_PAIRING_BE: u64 = 3;

/// Checks whether the product of pairings of a sequence of G1 and G2 points (in big-endian EIP-197 encoding)
/// on the BN254 curve evaluates to the identity element (1).
///
/// # Arguments
///
/// * `input` - A sequence of pairs of G1 and G2 points in big-endian (EIP-197) encoding.
///
/// # Returns
///
/// A `Result` containing:
/// - `Ok(true)` if the pairing product equals 1,
/// - `Ok(false)` otherwise,
/// - `Err(ProgramError)` if the input is invalid.
///
/// Note: This function does **not** check if the input has the correct length.
/// Currently, if the length is invalid, it will not return an error; instead it will use only
/// multiples of [`ALT_BN128_PAIRING_ELEMENT_SIZE`] bytes and discard the rest.
/// After SIMD-0334 is implemented, it will return an error if the length is invalid,
/// incurring the cost of the syscall.
#[inline(always)]
pub fn alt_bn128_is_pairing_valid_be(input: &[u8]) -> Result<bool, ProgramError> {
    alt_bn128_group_op::<32>(input, ALT_BN128_PAIRING_BE).map(|data| data[31] == 1)
}
