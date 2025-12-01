//! Multiplication operations on the BN254 curve.

use super::{alt_bn128_group_op, ALT_BN128_FIELD_SIZE, ALT_BN128_G1_POINT_SIZE};
use crate::program_error::ProgramError;

/// Input size for the multiplication operation.
pub const ALT_BN128_MULTIPLICATION_INPUT_SIZE: usize =
    ALT_BN128_G1_POINT_SIZE + ALT_BN128_FIELD_SIZE; // 96

/// Output size for the multiplication operation.
pub const ALT_BN128_MULTIPLICATION_OUTPUT_SIZE: usize = ALT_BN128_G1_POINT_SIZE; // 64

const ALT_BN128_G1_MUL_BE: u64 = 2;

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
#[inline(always)]
pub fn alt_bn128_g1_multiplication_be(
    input: &[u8; ALT_BN128_MULTIPLICATION_INPUT_SIZE],
) -> Result<[u8; ALT_BN128_MULTIPLICATION_OUTPUT_SIZE], ProgramError> {
    alt_bn128_group_op(input, ALT_BN128_G1_MUL_BE)
}
