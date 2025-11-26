//! # Pinocchio
//!
//! Pinocchio is a "no-external" dependencies library to create Solana programs
//! in Rust, which means that the only dependencies are from the Solana SDK. This
//! reduces the chance of dependency conflicts when compiling the program.
//!
//! It takes advantage of the way SVM loaders serialize the program input
//! parameters into a byte array that is then passed to the program's entrypoint
//! to use zero-copy types to read the input - these types are defined in an efficient
//! way taking into consideration that they will be used in on-chain programs.
//!
//! It is intended to be used by on-chain programs only; for off-chain programs,
//! use instead [`solana-sdk`] crates.
//!
//! [`solana-sdk`]: https://docs.rs/solana-sdk/latest/solana_sdk/
//!
//! ## Defining the program entrypoint
//!
//! A Solana program needs to define an entrypoint, which will be called by the
//! runtime to begin the program execution. The `entrypoint!` macro emits the common
//! boilerplate to set up the program entrypoint. The macro will also set up [global
//! allocator](https://doc.rust-lang.org/stable/core/alloc/trait.GlobalAlloc.html)
//! and [panic handler](https://doc.rust-lang.org/nomicon/panic-handler.html) using
//! the [`default_allocator!`] and [`default_panic_handler!`] macros.
//!
//! The [`entrypoint!`] is a convenience macro that invokes three other macros to set
//! all symbols required for a program execution:
//!
//! * [`program_entrypoint!`]: declares the program entrypoint
//! * [`default_allocator!`]: declares the default (bump) global allocator
//! * [`default_panic_handler!`]: declares the default panic handler
//!
//! To use the `entrypoint!` macro, use the following in your entrypoint definition:
//! ```ignore
//! use pinocchio::{
//!   AccountView,
//!   Address,
//!   entrypoint,
//!   ProgramResult
//! };
//! use solana_program_log::log;
//!
//! entrypoint!(process_instruction);
//!
//! pub fn process_instruction(
//!   program_id: &Address,
//!   accounts: &[AccountView],
//!   instruction_data: &[u8],
//! ) -> ProgramResult {
//!   log!("Hello from my pinocchio program!");
//!   Ok(())
//! }
//! ```
//!
//! The information from the input is parsed into their own entities:
//!
//! * `program_id`: the `ID` of the program being called
//! * `accounts`: the accounts received
//! * `instruction_data`: data for the instruction
//!
//! Pinocchio also offers variations of the program entrypoint
//! ([`lazy_program_entrypoint`]) and global allocator ([`no_allocator`]). In
//! order to use these, the program needs to specify the program entrypoint,
//! global allocator and panic handler individually. The [`entrypoint!`] macro
//! is equivalent to writing:
//! ```ignore
//! program_entrypoint!(process_instruction);
//! default_allocator!();
//! default_panic_handler!();
//! ```
//! Any of these macros can be replaced by other implementations and Pinocchio
//! offers a couple of variants for this.
//!
//! ### [`lazy_program_entrypoint!`]
//!
//! The [`entrypoint!`] macro looks similar to the "standard" one found in
//! [`solana-program-entrypoint`](https://docs.rs/solana-program-entrypoint/latest/solana_program_entrypoint/macro.entrypoint.html).
//! It parses the whole input and provides the `program_id`, `accounts` and
//! `instruction_data` separately. This consumes compute units before the program
//! begins its execution. In some cases, it is beneficial for a program to have
//! more control when the input parsing is happening, even whether the parsing
//! is needed or not - this is the purpose of the [`lazy_program_entrypoint!`]
//! macro. This macro only wraps the program input and provides methods to parse
//! the input on-demand.
//!
//! The [`lazy_program_entrypoint`] is suitable for programs that have a single
//! or very few instructions, since it requires the program to handle the parsing,
//! which can become complex as the number of instructions increases. For *larger*
//! programs, the [`program_entrypoint!`] will likely be easier and more efficient
//! to use.
//!
//! To use the [`lazy_program_entrypoint!`] macro, use the following in your
//! entrypoint definition:
//! ```ignore
//! use pinocchio::{
//!   default_allocator,
//!   default_panic_handler,
//!   entrypoint::InstructionContext,
//!   lazy_program_entrypoint,
//!   ProgramResult
//! };
//!
//! lazy_program_entrypoint!(process_instruction);
//! default_allocator!();
//! default_panic_handler!();
//!
//! pub fn process_instruction(
//!   mut context: InstructionContext
//! ) -> ProgramResult {
//!   Ok(())
//! }
//! ```
//!
//! The [`InstructionContext`](entrypoint::InstructionContext) provides on-demand
//! access to the information of the input:
//!
//! * [`remaining()`](entrypoint::InstructionContext::remaining): number of available
//!   accounts to parse; this number is decremented as the program parses accounts.
//! * [`next_account()`](entrypoint::InstructionContext::next_account): parses the
//!   next available account (can be used as many times as accounts available).
//! * [`instruction_data()`](entrypoint::InstructionContext::instruction_data): parses
//!   the instruction data.
//! * [`program_id()`](entrypoint::InstructionContext::program_id): parses the
//!   program id.
//!
//!
//! 💡 The [`lazy_program_entrypoint!`] does not set up a global allocator nor a panic
//! handler. A program should explicitly use one of the provided macros to set them
//! up or include its own implementation.
//!
//! ### [`no_allocator!`]
//!
//! When writing programs, it can be useful to make sure the program does not attempt
//! to make any allocations. For this cases, Pinocchio includes a [`no_allocator!`]
//! macro that set a global allocator just panics at any attempt to allocate memory.
//!
//! To use the [`no_allocator!`] macro, use the following in your entrypoint definition:
//! ```ignore
//! use pinocchio::{
//!   AccountView,
//!   Address,
//!   default_panic_handler,
//!   no_allocator,
//!   program_entrypoint,
//!   ProgramResult
//! };
//!
//! program_entrypoint!(process_instruction);
//! default_panic_handler!();
//! no_allocator!();
//!
//! pub fn process_instruction(
//!   program_id: &Address,
//!   accounts: &[AccountView],
//!   instruction_data: &[u8],
//! ) -> ProgramResult {
//!   Ok(())
//! }
//! ```
//!
//!
//! 💡 The [`no_allocator!`] macro can also be used in combination with the
//! [`lazy_program_entrypoint!`].
//!
//! ## `alloc` crate feature
//!
//! By default, Pinocchio is a `no_std` crate. This means that it does not use any
//! code from the standard (`std`) library. While this does not affect how Pinocchio
//! is used, there is a one particular apparent difference. Helpers that need to
//! allocate memory, such as fetching `SlotHashes` sysvar, are not available. To
//! enable these helpers, the `alloc` feature must be enabled when adding Pinocchio
//! as a dependency:
//! ```ignore
//! pinocchio = { version = "0.10.0", features = ["alloc"] }
//! ```
//!
//! ## Advanced entrypoint configuration
//!
//! The symbols emitted by the entrypoint macros - program entrypoint, global
//! allocator and default panic handler - can only be defined once globally. If
//! the program crate is also intended to be used as a library, it is common practice
//! to define a Cargo [feature](https://doc.rust-lang.org/cargo/reference/features.html)
//! in your program crate to conditionally enable the module that includes the [`entrypoint!`]
//! macro invocation. The convention is to name the feature `bpf-entrypoint`.
//!
//! ```ignore
//! #[cfg(feature = "bpf-entrypoint")]
//! mod entrypoint {
//!   use pinocchio::{
//!     AccountView,
//!     Address,
//!     entrypoint,
//!     ProgramResult
//!   };
//!
//!   entrypoint!(process_instruction);
//!
//!   pub fn process_instruction(
//!     program_id: &Address,
//!     accounts: &[AccountView],
//!     instruction_data: &[u8],
//!   ) -> ProgramResult {
//!     Ok(())
//!   }
//! }
//! ```
//!
//! When building the program binary, you must enable the `bpf-entrypoint` feature:
//! ```ignore
//! cargo build-sbf --features bpf-entrypoint
//! ```

#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod entrypoint;
pub mod sysvars;

#[deprecated(since = "0.7.0", note = "Use the `entrypoint` module instead")]
pub use entrypoint::lazy as lazy_entrypoint;

// Re-export the `solana_account_view` for downstream use.
pub use solana_account_view as account;
pub use solana_account_view::AccountView;

// Re-export the `solana_address` for downstream use.
pub use solana_address as address;
pub use solana_address::Address;

// Re-export the `solana_define_syscall` for downstream use.
#[cfg(any(target_os = "solana", target_arch = "bpf"))]
pub use solana_define_syscall::definitions as syscalls;

// Re-export the `solana_instruction_view` for downstream use.
#[cfg(feature = "cpi")]
pub use {solana_instruction_view as instruction, solana_instruction_view::cpi};

// Re-export the `solana_program_error` for downstream use.
pub use solana_program_error as error;
pub use solana_program_error::ProgramResult;

/// Maximum number of accounts that a transaction may process.
///
/// This value is set to `u8::MAX - 1`, which is the theoretical maximum
/// number of accounts that a transaction can process given that indices
/// of accounts are represented by an `u8` value and the last
/// value (`255`) is reserved to indicate non-duplicated accounts.
///
/// The `MAX_TX_ACCOUNTS` is used to statically initialize the array of
/// `AccountView`s when parsing accounts in an instruction.
pub const MAX_TX_ACCOUNTS: usize = (u8::MAX - 1) as usize;

/// `assert_eq(core::mem::align_of::<u128>(), 8)` is true for BPF but not
/// for some host machines.
const BPF_ALIGN_OF_U128: usize = 8;

/// Return value for a successful program execution.
pub const SUCCESS: u64 = 0;

/// Module with functions to provide hints to the compiler about how code
/// should be optimized.
pub mod hint {
    /// A "dummy" function with a hint to the compiler that it is unlikely to be
    /// called.
    ///
    /// This function is used as a hint to the compiler to optimize other code paths
    /// instead of the one where the function is used.
    #[cold]
    pub const fn cold_path() {}

    /// Return the given `bool` value with a hint to the compiler that `true` is the
    /// likely case.
    #[inline(always)]
    pub const fn likely(b: bool) -> bool {
        if b {
            true
        } else {
            cold_path();
            false
        }
    }

    /// Return a given `bool` value with a hint to the compiler that `false` is the
    /// likely case.
    #[inline(always)]
    pub const fn unlikely(b: bool) -> bool {
        if b {
            cold_path();
            true
        } else {
            false
        }
    }
}
