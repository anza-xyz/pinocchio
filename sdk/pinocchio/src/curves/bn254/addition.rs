//! Addition operations on the BN254 curve.

use super::{alt_bn128_group_op, ALT_BN128_G1_POINT_SIZE};
use crate::program_error::ProgramError;

/// Input size for the add operation.
pub const ALT_BN128_ADDITION_INPUT_SIZE: usize = ALT_BN128_G1_POINT_SIZE * 2; // 128

/// Output size for the add operation.
pub const ALT_BN128_ADDITION_OUTPUT_SIZE: usize = ALT_BN128_G1_POINT_SIZE; // 64

const ALT_BN128_G1_ADD_BE: u64 = 0;
#[allow(dead_code)]
const ALT_BN128_G1_SUB_BE: u64 = 1; // not implemented in the syscall

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
pub fn alt_bn128_g1_addition_be(
    input: &[u8; ALT_BN128_ADDITION_INPUT_SIZE],
) -> Result<[u8; ALT_BN128_ADDITION_OUTPUT_SIZE], ProgramError> {
    alt_bn128_group_op(input, ALT_BN128_G1_ADD_BE)
}
