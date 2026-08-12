//! Public key recovery from [secp256k1] ECDSA signatures.
//!
//! [secp256k1]: https://en.bitcoin.it/wiki/Secp256k1

/// Length of a secp256k1 ECDSA signature.
pub const SECP256K1_SIGNATURE_LENGTH: usize = 64;

/// Length of a secp256k1 public key.
pub const SECP256K1_PUBLIC_KEY_LENGTH: usize = 64;

/// Length of a message hash for secp256k1 recovery.
pub const SECP256K1_MESSAGE_HASH_LENGTH: usize = 32;

/// Error returned by [`secp256k1_recover`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum Secp256k1RecoverError {
    /// The hash provided to a secp256k1_recover is invalid.
    InvalidHash = 1,
    /// The recovery_id provided to a secp256k1_recover is invalid.
    InvalidRecoveryId = 2,
    /// The signature provided to a secp256k1_recover is invalid.
    InvalidSignature = 3,
}

/// Recovers a secp256k1 public key from a signed message.
///
/// Given a signed message, the signature, and a recovery ID, this function
/// recovers the secp256k1 public key that was used to sign the message.
///
/// # Arguments
///
/// * `hash` - The 32-byte message hash that was signed.
/// * `recovery_id` - The recovery ID (0-3) generated during signing, as defined by [k256](https://docs.rs/k256/0.13.4/k256/ecdsa/struct.RecoveryId.html).
/// * `signature` - The 64-byte ECDSA signature (r and s components concatenated).
///
/// # Returns
///
/// Returns the recovered public key on success, or a
/// [`Secp256k1RecoverError`] on failure.
#[inline(always)]
pub fn secp256k1_recover(
    hash: &[u8; SECP256K1_MESSAGE_HASH_LENGTH],
    recovery_id: u8,
    signature: &[u8; SECP256K1_SIGNATURE_LENGTH],
) -> Result<[u8; SECP256K1_PUBLIC_KEY_LENGTH], Secp256k1RecoverError> {
    #[cfg(any(target_os = "solana", target_arch = "bpf"))]
    {
        let mut pubkey_buffer =
            core::mem::MaybeUninit::<[u8; SECP256K1_PUBLIC_KEY_LENGTH]>::uninit();
        let result = unsafe {
            crate::syscalls::sol_secp256k1_recover(
                hash.as_ptr(),
                recovery_id as u64,
                signature.as_ptr(),
                pubkey_buffer.as_mut_ptr() as *mut u8,
            )
        };
        match result {
            0 => Ok(unsafe { pubkey_buffer.assume_init() }),
            _ => Err(unsafe { core::mem::transmute::<u64, Secp256k1RecoverError>(result) }),
        }
    }

    #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
    {
        core::hint::black_box((hash, recovery_id, signature));
        panic!("secp256k1_recover is only available on target `solana`")
    }
}
