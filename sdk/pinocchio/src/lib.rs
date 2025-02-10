//! A library to build a Solana program in Rust.
//!
//! This library is intended to be used by on-chain programs only. It provides
//! a zero-dependency library to minimise dependencies conflits. For off-chain
//! programs, use instead the [`solana-sdk`] crate, which re-exports all modules
//! from [`solana-program`].
//!
//! [`solana-sdk`]: https://docs.rs/solana-sdk/latest/solana_sdk/
//! [`solana-program`]: https://docs.rs/solana-program/latest/solana_program/

#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod account_info;
pub mod entrypoint;
pub mod instruction;
pub mod log;
pub mod memory;
pub mod program_error;
pub mod pubkey;
pub mod syscalls;
pub mod sysvars;

#[deprecated(since = "0.7.0", note = "Use the `entrypoint` module instead")]
pub use entrypoint::lazy as lazy_entrypoint;

/// Maximum number of accounts that a transaction may process.
///
/// This value is used to set the maximum number of accounts that a program
/// is expecting and statically initialize the array of `AccountInfo`.
///
/// This is based on the current [maximum number of accounts] that a transaction
/// may lock in a block.
///
/// [maximum number of accounts]: https://github.com/anza-xyz/agave/blob/2e6ca8c1f62db62c1db7f19c9962d4db43d0d550/runtime/src/bank.rs#L3209-L3221
pub const MAX_TX_ACCOUNTS: usize = 128;

/// `assert_eq(core::mem::align_of::<u128>(), 8)` is true for BPF but not
/// for some host machines.
const BPF_ALIGN_OF_U128: usize = 8;

/// Value used to indicate that a serialized account is not a duplicate.
const NON_DUP_MARKER: u8 = u8::MAX;

/// Return value for a successful program execution.
pub const SUCCESS: u64 = 0;

/// The result of a program execution.
pub type ProgramResult = Result<(), program_error::ProgramError>;
