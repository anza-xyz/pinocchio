#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! A *no external* dependencies library to create Solana programs
//! in Rust. The only dependencies are types from the Solana SDK specifically
//! designed for on-chain programs. This mitigates dependency issues and offers
//! an efficient zero-copy library to write programs, optimized in terms of both
//! compute units consumption and binary size.
//!
//! ## Defining the program entrypoint
//!
//! A Solana program needs to define an entrypoint, which will be called by the
//! runtime to begin the program execution. The `entrypoint!` macro emits the
//! common boilerplate to set up the program entrypoint. The macro will also set
//! up [global allocator](https://doc.rust-lang.org/stable/core/alloc/trait.GlobalAlloc.html)
//! and [panic handler](https://doc.rust-lang.org/nomicon/panic-handler.html) using
//! the [`default_allocator!`] and [`default_panic_handler!`] macros.
//!
//! The [`entrypoint!`](https://docs.rs/pinocchio/latest/pinocchio/macro.entrypoint.html)
//! is a convenience macro that invokes three other macros to set all components
//! required for a program execution:
//!
//! * [`program_entrypoint!`]: declares the program entrypoint
//! * [`default_allocator!`]: declares the default (bump) global allocator
//! * [`default_panic_handler!`]: declares the default panic "hook" that works
//!   in combination with the `std` panic handler
//!
//! When all dependencies are `no_std`, you should use [`nostd_panic_handler!`](https://docs.rs/pinocchio/latest/pinocchio/macro.nostd_panic_handler.html)
//! instead of `default_panic_handler!` to declare a rust runtime panic handler.
//! There's no need to do this when any dependency is `std` since rust compiler
//! will emit a panic handler.
//!
//! To use the `entrypoint!` macro, use the following in your entrypoint
//! definition:
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
//! `pinocchio` also offers variations of the program entrypoint
//! (`lazy_program_entrypoint`) and global allocator (`no_allocator`). In order
//! to use these, the program needs to specify the program entrypoint, global
//! allocator and panic handler individually. The `entrypoint!` macro is
//! equivalent to writing:
//! ```ignore
//! program_entrypoint!(process_instruction);
//! default_allocator!();
//! default_panic_handler!();
//! ```
//! Any of these macros can be replaced by alternative implementations.
//!
//! ### Custom entrypoints with [`crate::entrypoint::process_entrypoint`]
//!
//! For programs that need maximum control over the entrypoint, `pinocchio`
//! exposes the [`crate::entrypoint::process_entrypoint`] function. This
//! function is the same deserialization logic used internally by the
//! [`program_entrypoint!`] macro, exposed as a public API and can be called
//! directly from a custom entrypoint, allowing you to implement fast-path
//! optimizations or custom pre-processing logic before falling back to standard
//! input parsing.
//!
//! To use [`crate::entrypoint::process_entrypoint`] in a custom entrypoint:
//!
//! ```ignore
//! use pinocchio::{
//!   AccountView,
//!   Address,
//!   default_panic_handler,
//!   entrypoint::process_entrypoint,
//!   MAX_TX_ACCOUNTS,
//!   no_allocator,
//!   ProgramResult,
//! };
//! use solana_program_log::log;
//!
//! no_allocator!();
//! default_panic_handler!();
//!
//! #[no_mangle]
//! pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
//!   // Fast path: check the number of accounts
//!   let num_accounts = unsafe { *(input as *const u64) };
//!   if num_accounts == 0 {
//!     log("Fast path - no accounts!");
//!     return 0;
//!   }
//!
//!   // Standard path: delegate to `process_entrypoint`
//!   unsafe { process_entrypoint::<MAX_TX_ACCOUNTS>(input, process_instruction) }
//! }
//!
//! pub fn process_instruction(
//!   program_id: &Address,
//!   accounts: &[AccountView],
//!   instruction_data: &[u8],
//! ) -> ProgramResult {
//!   log("Standard path");
//!   Ok(())
//! }
//! ```
//!
//! ### [`lazy_program_entrypoint!`]
//!
//! The [`entrypoint!`] macro looks similar to the "standard" one found in
//! [`solana-program-entrypoint`](https://docs.rs/solana-program-entrypoint/latest/solana_program_entrypoint/macro.entrypoint.html).
//! It parses the whole input and provides the `program_id`, `accounts` and
//! `instruction_data` separately. This consumes compute units before the
//! program begins its execution. In some cases, it is beneficial for a program
//! to have more control when the input parsing is happening, even whether the
//! parsing is needed or not - this is the purpose of the
//! [`lazy_program_entrypoint!`] macro. This macro only wraps the program input
//! and provides methods to parse the input on-demand.
//!
//! The [`lazy_program_entrypoint`] is suitable for programs that have a single
//! or very few instructions, since it requires the program to handle the
//! parsing, which can become complex as the number of instructions increases.
//! For *larger* programs, the [`program_entrypoint!`] will likely be easier and
//! more efficient to use.
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
//! The [`InstructionContext`](entrypoint::InstructionContext) provides
//! on-demand access to the information of the input:
//!
//! * [`remaining()`](entrypoint::InstructionContext::remaining): number of
//!   available accounts to parse; this number is decremented as the program
//!   parses accounts.
//! * [`next_account()`](entrypoint::InstructionContext::next_account): parses
//!   the next available account (can be used as many times as accounts
//!   available).
//! * [`instruction_data()`](entrypoint::InstructionContext::instruction_data):
//!   parses the instruction data.
//! * [`program_id()`](entrypoint::InstructionContext::program_id): parses the
//!   program id.
//!
//!
//! 💡 The [`lazy_program_entrypoint!`] does not set up a global allocator nor a
//! panic handler. A program should explicitly use one of the provided macros to
//! set them up or include its own implementation.
//!
//! ### [`no_allocator!`]
//!
//! When writing programs, it can be useful to make sure the program does not
//! attempt to make any allocations. For this cases, Pinocchio includes a
//! [`no_allocator!`] macro that set a global allocator just panics at any
//! attempt to allocate memory.
//!
//! To use the [`no_allocator!`] macro, use the following in your entrypoint
//! definition:
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
//! 💡 The [`no_allocator!`] macro can also be used in combination with the
//! [`lazy_program_entrypoint!`].
//!
//! Since the `no_allocator!` macro does not allocate memory, the `32kb` memory
//! region reserved for the heap remains unused. To take advantage of this, the
//! `no_allocator!` macro emits an `allocate_unchecked` helper function that
//! allows you to manually reserve memory for a type at compile time.
//! ```ignore
//! // static allocation:
//! //    - 0 is the offset when the type will be allocated
//! //    - `allocate_unchecked` returns a mutable reference to the allocated
//! type let lamports = allocate_unchecked::<u64>(0);
//! *lamports = 1_000_000_000;
//! ```
//!
//! Note that it is the developer's responsibility to ensure that types do not
//! overlap in memory - the `offset + <size of type>` of different types must
//! not overlap.
//!
//! ### [`nostd_panic_handler!`]
//!
//! When writing `no_std` programs, it is necessary to declare a panic handler
//! using the [`nostd_panic_handler!`] macro. This macro sets up a default panic
//! handler that logs the location (file, line and column) where the panic
//! occurred and then calls the `abort()` syscall.
//!
//! 💡 The `default_panic_handler!` macro only works in an `std` context.
//!
//! ## Crate features
//!
//! ### `alloc`
//!
//! The `alloc` feature is enabled by default and it uses the [`alloc`](https://doc.rust-lang.org/alloc/)
//! crate. This provides access to dynamic memory allocation in combination with
//! the [`default_allocator!`], e.g., required to use `String` and `Vec` in a
//! program. Helpers that need to allocate memory, such as fetching
//! [`crate::sysvars::slot_hashes::SlotHashes::fetch`] sysvar data, are also
//! available.
//!
//! When no allocation is needed or desired, the feature can be disabled:
//! ```ignore
//! pinocchio = { version = "0.10.0", default-features = false }
//! ```
//!
//! ### `copy`
//!
//! The `copy` feature enables the derivation of the `Copy` trait for types. It
//! also enables the `copy` feature on the `solana-account-view` and
//! `solana-address` re-exports.
//!
//! ### `cpi`
//!
//! The `cpi` feature enables the cross-program invocation helpers, as well as
//! types to define instructions and signer information.
//! ```ignore
//! pinocchio = { version = "0.10.0", features = ["cpi"] }
//! ```
//!
//! ## Advanced entrypoint configuration
//!
//! The symbols emitted by the entrypoint macros - program entrypoint, global
//! allocator and default panic handler - can only be defined once globally. If
//! the program crate is also intended to be used as a library, it is common
//! practice to define a Cargo [feature](https://doc.rust-lang.org/cargo/reference/features.html)
//! in your program crate to conditionally enable the module that includes the
//! [`entrypoint!`] macro invocation. The convention is to name the feature
//! `bpf-entrypoint`.
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
//! When building the program binary, you must enable the `bpf-entrypoint`
//! feature:
//! ```ignore
//! cargo build-sbf --features bpf-entrypoint
//! ```
//!
//! ## Upstream BPF compatibility
//!
//! Pinocchio is compatible with upstream BPF target (`target_arch = bpf`). When
//! using syscalls (e.g., cross-program invocations), it is necessary to
//! explicitly enable static syscalls in your program's `Cargo.toml`:
//! ```toml
//! [dependencies]
//! # Enable static syscalls for BPF target
//! solana-define-syscall = { version = "4.0.1", features = ["unstable-static-syscalls"] }
//! ```
//!
//! When compiling your program with the upstream BPF target, the `std` library
//! is not available. Therefore, the program crate must include the `#![no_std]`
//! crate-level attribute and use the [`nostd_panic_handler!`] macro. An
//! allocator may be used as long as `alloc` is used.

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod entrypoint;
pub mod sysvars;

// Re-export the `solana_define_syscall` for downstream use.
#[cfg(any(target_os = "solana", target_arch = "bpf"))]
pub use solana_define_syscall::definitions as syscalls;
#[cfg(feature = "resize")]
use {
    core::ptr::write_bytes,
    solana_account_view::{RuntimeAccount, MAX_PERMITTED_DATA_INCREASE},
    solana_program_error::ProgramError,
};
// Re-export for downstream use:
//   - `solana_account_view`
//   - `solana_address`
//   - `solana_program_error`
pub use {
    solana_account_view::{self as account, AccountView},
    solana_address::{self as address, Address},
    solana_program_error::{self as error, ProgramResult},
};
// Re-export the `solana_instruction_view` for downstream use.
#[cfg(feature = "cpi")]
pub use {solana_instruction_view as instruction, solana_instruction_view::cpi};

/// Maximum number of accounts that a transaction may process.
///
/// This value is set to `u8::MAX`, which is the theoretical maximum
/// number of accounts that a transaction can process given that indices
/// of accounts are represented by an `u8` value and the last
/// value (`255`) is reserved to indicate non-duplicated accounts.
///
/// The `MAX_TX_ACCOUNTS` is used to statically initialize the array of
/// `AccountView`s when parsing accounts in an instruction.
pub const MAX_TX_ACCOUNTS: usize = u8::MAX as usize;

/// `assert_eq(core::mem::align_of::<u128>(), 8)` is true for BPF but not
/// for some host machines.
const BPF_ALIGN_OF_U128: usize = 8;

/// Return value for a successful program execution.
pub const SUCCESS: u64 = 0;

#[cfg(feature = "resize")]
/// Trait to indicate that an account can be resized.
pub trait Resize: sealed::Sealed {
    /// Resize (either truncating or zero extending) the account's data.
    ///
    /// The account data can be increased by up to
    /// [`MAX_PERMITTED_DATA_INCREASE`] bytes within an instruction.
    ///
    /// # Important
    ///
    /// This method makes assumptions about the layout and location of memory
    /// referenced by `RuntimeAccount` fields. It should only be called for
    /// instances of `AccountView` that were created by the runtime and received
    /// in the `process_instruction` entrypoint of a program.
    fn resize(&self, new_len: usize) -> Result<(), ProgramError>;

    /// Resize (either truncating or zero extending) the account's data.
    ///
    /// The account data can be increased by up to
    /// [`MAX_PERMITTED_DATA_INCREASE`] bytes within an instruction.
    ///
    /// # Important
    ///
    /// This method makes assumptions about the layout and location of memory
    /// referenced by `RuntimeAccount` fields. It should only be called for
    /// instances of `AccountView` that were created by the runtime and received
    /// in the `process_instruction` entrypoint of a program.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it does not check if the account data is
    /// already borrowed. The caller must guarantee that there are no active
    /// borrows to the account data.
    unsafe fn resize_unchecked(&self, new_len: usize) -> Result<(), ProgramError>;
}

// Only AccountView is "sealed", so only it can implement `Resize`.
#[cfg(feature = "resize")]
impl sealed::Sealed for AccountView {}

#[cfg(feature = "resize")]
impl Resize for AccountView {
    fn resize(&self, new_len: usize) -> Result<(), ProgramError> {
        // Check whether the account data is already borrowed.
        self.check_borrow_mut()?;

        unsafe { self.resize_unchecked(new_len) }
    }

    unsafe fn resize_unchecked(&self, new_len: usize) -> Result<(), ProgramError> {
        // Return early if length hasn't changed.
        if new_len == self.data_len() {
            return Ok(());
        }

        let account = self.account_ptr() as *mut RuntimeAccount;
        let original_data_len = unsafe { (*account).padding as usize };
        // Return early if the length increase from the original serialized data
        // length is too large and would result in an out-of-bounds allocation.
        if new_len.saturating_sub(original_data_len) > MAX_PERMITTED_DATA_INCREASE {
            return Err(ProgramError::InvalidRealloc);
        }

        if new_len > self.data_len() {
            unsafe {
                write_bytes(
                    self.data_ptr().add(self.data_len()),
                    0,
                    new_len.saturating_sub(self.data_len()),
                );
            }
        }

        unsafe { (*account).data_len = new_len as u64 };

        Ok(())
    }
}

// Not `pub`, so other crates cannot name nor implement its trait.
#[cfg(feature = "resize")]
mod sealed {
    // This is used to "seal" the `Resize` trait, preventing external crates from
    // implementing it for types other than `AccountView`.
    pub trait Sealed {}
}

/// Module with functions to provide hints to the compiler about how code
/// should be optimized.
pub mod hint {
    /// A "dummy" function with a hint to the compiler that it is unlikely to be
    /// called.
    ///
    /// This function is used as a hint to the compiler to optimize other code
    /// paths instead of the one where the function is used.
    #[cold]
    pub const fn cold_path() {}

    /// Return the given `bool` value with a hint to the compiler that `true` is
    /// the likely case.
    #[inline(always)]
    pub const fn likely(b: bool) -> bool {
        if b {
            true
        } else {
            cold_path();
            false
        }
    }

    /// Return a given `bool` value with a hint to the compiler that `false` is
    /// the likely case.
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

#[cfg(test)]
mod tests {
    use {super::*, solana_account_view::NOT_BORROWED};
    #[test]
    fn test_resize() {
        struct TestAccount {
            _raw: *mut RuntimeAccount,
        }

        // 8-bytes aligned account data.
        let mut data = [0u64; 100 * size_of::<u64>()];

        // Set the borrow state.
        data[0] = NOT_BORROWED as u64;
        // Set the initial data length to 100.
        //
        // (dup_marker, signer, writable, executable and padding)
        data[0] = u64::from_le_bytes([255, 0, 0, 0, 100, 0, 0, 0]);
        // `data_len`
        data[10] = 100;

        let runtime_account = data.as_mut_ptr() as *mut RuntimeAccount;

        let test_account = TestAccount {
            _raw: runtime_account,
        };
        // SAFETY: A reference to `AccountView` that points to the same memory
        // as `runtime_account`.
        let account = unsafe { &*(&test_account as *const _ as *const AccountView) };

        assert_eq!(account.data_len(), 100);
        assert_eq!(unsafe { (*runtime_account).padding }, 100);

        // We should be able to get the data pointer whenever as long as we don't use
        // it while the data is borrowed.
        let data_ptr_before = account.data_ptr();

        // increase the size.

        account.resize(200).unwrap();

        let data_ptr_after = account.data_ptr();
        // The data pointer should point to the same address regardless of the
        // reallocation
        assert_eq!(data_ptr_before, data_ptr_after);

        assert_eq!(account.data_len(), 200);

        // decrease the size.

        account.resize(0).unwrap();

        assert_eq!(account.data_len(), 0);

        // Invalid reallocation.

        let invalid_realloc = account.resize(10_000_000_001);
        assert!(invalid_realloc.is_err());

        // Reset to its original size.

        account.resize(100).unwrap();

        assert_eq!(account.data_len(), 100);

        // Consecutive resizes.

        account.resize(200).unwrap();
        account.resize(50).unwrap();
        account.resize(500).unwrap();

        assert_eq!(account.data_len(), 500);

        let data = account.try_borrow().unwrap();
        assert_eq!(data.len(), 500);
    }
}
