//! Compression/decompression of points on the BN254 curve.

use super::{ALT_BN128_G1_POINT_SIZE, ALT_BN128_G2_POINT_SIZE};
use crate::program_error::ProgramError;

#[cfg(target_os = "solana")]
use crate::syscalls::sol_alt_bn128_compression;

// compression sizes
pub const ALT_BN128_G1_COMPRESSED_POINT_SIZE: usize = ALT_BN128_G1_POINT_SIZE / 2; // 32
pub const ALT_BN128_G2_COMPRESSED_POINT_SIZE: usize = ALT_BN128_G2_POINT_SIZE / 2; // 64

// compression operations
const ALT_BN128_G1_COMPRESS_BE: u64 = 0;
const ALT_BN128_G1_DECOMPRESS_BE: u64 = 1;
const ALT_BN128_G2_COMPRESS_BE: u64 = 2;
const ALT_BN128_G2_DECOMPRESS_BE: u64 = 3;

/// Compress a G1 point on the BN254 curve.
///
/// # Arguments
///
/// * `input` - A G1 point in big-endian (EIP-197) encoding.
///
/// # Returns
///
/// A `Result` containing the compressed G1 point in big-endian (EIP-197) encoding,
/// or an error if the input is not a valid G1 point.
#[inline(always)]
pub fn alt_bn128_g1_compress_be(
    input: &[u8; ALT_BN128_G1_POINT_SIZE],
) -> Result<[u8; ALT_BN128_G1_COMPRESSED_POINT_SIZE], ProgramError> {
    alt_bn128_compression(input, ALT_BN128_G1_COMPRESS_BE)
}

/// Decompress a G1 point on the BN254 curve.
///
/// # Arguments
///
/// * `input` - A compressed G1 point in big-endian (EIP-197) encoding.
///
/// # Returns
///
/// A `Result` containing the decompressed G1 point in big-endian (EIP-197) encoding,
/// or an error if the input is not a valid compressed G1 point.
#[inline(always)]
pub fn alt_bn128_g1_decompress_be(
    input: &[u8; ALT_BN128_G1_COMPRESSED_POINT_SIZE],
) -> Result<[u8; ALT_BN128_G1_POINT_SIZE], ProgramError> {
    alt_bn128_compression(input, ALT_BN128_G1_DECOMPRESS_BE)
}

/// Compress a G2 point on the BN254 curve.
///
/// # Arguments
///
/// * `input` - A G2 point in big-endian (EIP-197) encoding.
///
/// # Returns
///
/// A `Result` containing the compressed G2 point in big-endian (EIP-197) encoding,
/// or an error if the input is not a valid G2 point.
#[inline(always)]
pub fn alt_bn128_g2_compress_be(
    input: &[u8; ALT_BN128_G2_POINT_SIZE],
) -> Result<[u8; ALT_BN128_G2_COMPRESSED_POINT_SIZE], ProgramError> {
    alt_bn128_compression(input, ALT_BN128_G2_COMPRESS_BE)
}

/// Decompress a G2 point on the BN254 curve.
///
/// # Arguments
///
/// * `input` - A compressed G2 point in big-endian (EIP-197) encoding.
///
/// # Returns
///
/// A `Result` containing the decompressed G2 point in big-endian (EIP-197) encoding,
/// or an error if the input is not a valid compressed G2 point.
#[inline(always)]
pub fn alt_bn128_g2_decompress_be(
    input: &[u8; ALT_BN128_G2_COMPRESSED_POINT_SIZE],
) -> Result<[u8; ALT_BN128_G2_POINT_SIZE], ProgramError> {
    alt_bn128_compression(input, ALT_BN128_G2_DECOMPRESS_BE)
}

#[inline]
fn alt_bn128_compression<const OUTPUT_DATA_SIZE: usize>(
    input: &[u8],
    op: u64,
) -> Result<[u8; OUTPUT_DATA_SIZE], ProgramError> {
    // Call via a system call to perform the calculation
    #[cfg(target_os = "solana")]
    {
        let mut bytes = core::mem::MaybeUninit::<[u8; OUTPUT_DATA_SIZE]>::uninit();

        let result = unsafe {
            sol_alt_bn128_compression(
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
        panic!("alt_bn128_compression is only available on target `solana`")
    }
}
