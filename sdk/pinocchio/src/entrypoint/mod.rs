//! Macros and functions for defining the program entrypoint and setting up
//! global handlers.

pub mod lazy;

pub use lazy::{InstructionContext, MaybeAccount};

#[cfg(target_os = "solana")]
pub use alloc::BumpAllocator;
use core::{
    cmp::min,
    mem::{size_of, MaybeUninit},
    slice::from_raw_parts,
};

use crate::{
    account_info::{Account, AccountInfo, MAX_PERMITTED_DATA_INCREASE},
    pubkey::Pubkey,
    BPF_ALIGN_OF_U128, MAX_TX_ACCOUNTS, NON_DUP_MARKER,
};

/// Start address of the memory region used for program heap.
pub const HEAP_START_ADDRESS: u64 = 0x300000000;

/// Length of the heap memory region used for program heap.
pub const HEAP_LENGTH: usize = 32 * 1024;

#[deprecated(
    since = "0.6.0",
    note = "Use `ProgramResult` from the crate root instead"
)]
/// The result of a program execution.
pub type ProgramResult = super::ProgramResult;

#[deprecated(since = "0.6.0", note = "Use `SUCCESS` from the crate root instead")]
/// Return value for a successful program execution.
pub const SUCCESS: u64 = super::SUCCESS;

/// The "static" size of an account in the input buffer.
///
/// This is the size of the account header plus the maximum permitted data increase.
const STATIC_ACCOUNT_DATA: usize = size_of::<Account>() + MAX_PERMITTED_DATA_INCREASE;

/// Declare the program entrypoint and set up global handlers.
///
/// The main difference from the standard (SDK) [`entrypoint`](https://docs.rs/solana-program-entrypoint/latest/solana_program_entrypoint/macro.entrypoint.html)
/// macro is that this macro represents an entrypoint that does not perform allocations or copies
/// when reading the input buffer.
///
/// This macro emits the common boilerplate necessary to begin program execution, calling a
/// provided function to process the program instruction supplied by the runtime, and reporting
/// its result to the runtime.
///
/// It also sets up a [global allocator] and [panic handler], using the [`crate::default_allocator!`]
/// and [`crate::default_panic_handler!`] macros.
///
/// The first argument is the name of a function with this type signature:
///
/// ```ignore
/// fn process_instruction(
///     program_id: &Pubkey,      // Public key of the account the program was loaded into
///     accounts: &[AccountInfo], // All accounts required to process the instruction
///     instruction_data: &[u8],  // Serialized instruction-specific data
/// ) -> ProgramResult;
/// ```
/// The argument is defined as an `expr`, which allows the use of any function pointer not just
/// identifiers in the current scope.
///
/// There is a second optional argument that allows to specify the maximum number of accounts
/// expected by instructions of the program. This is useful to reduce the stack size requirement
/// for the entrypoint, as the default is set to [`crate::MAX_TX_ACCOUNTS`]. If the program
/// receives more accounts than the specified maximum, these accounts will be ignored.
///
/// [global allocator]: https://doc.rust-lang.org/stable/alloc/alloc/trait.GlobalAlloc.html
/// [maximum number of accounts]: https://github.com/anza-xyz/agave/blob/ccabfcf84921977202fd06d3197cbcea83742133/runtime/src/bank.rs#L3207-L3219
/// [panic handler]: https://doc.rust-lang.org/stable/core/panic/trait.PanicHandler.html
///
/// # Examples
///
/// Defining an entrypoint conditional on the `bpf-entrypoint` feature. Although the `entrypoint`
/// module is written inline in this example, it is common to put it into its own file.
///
/// ```no_run
/// #[cfg(feature = "bpf-entrypoint")]
/// pub mod entrypoint {
///
///     use pinocchio::{
///         account_info::AccountInfo,
///         entrypoint,
///         msg,
///         pubkey::Pubkey,
///         ProgramResult
///     };
///
///     entrypoint!(process_instruction);
///
///     pub fn process_instruction(
///         program_id: &Pubkey,
///         accounts: &[AccountInfo],
///         instruction_data: &[u8],
///     ) -> ProgramResult {
///         msg!("Hello from my program!");
///         Ok(())
///     }
///
/// }
/// ```
///
/// # Important
///
/// The panic handler set up is different depending on whether the `std` library is available to the
/// linker or not. The `entrypoint` macro will set up a default
/// panic "hook", that works with the `#[panic_handler]` set by the `std`. Therefore, this macro
/// should be used when the program or any of its dependencies are dependent on the `std` library.
///
/// When the program and all its dependencies are `no_std`, it is necessary to set a
/// `#[panic_handler]` to handle panics. This is done by the [`crate::nostd_panic_handler`](https://docs.rs/pinocchio/latest/pinocchio/macro.nostd_panic_handler.html)
/// macro. In this case, it is not possible to use the `entrypoint`
/// macro. Use the [`crate::program_entrypoint!`] macro instead and set up the allocator and panic
/// handler manually.
#[macro_export]
macro_rules! entrypoint {
    ( $process_instruction:expr ) => {
        $crate::entrypoint!($process_instruction, { $crate::MAX_TX_ACCOUNTS });
    };
    ( $process_instruction:expr, $maximum:expr ) => {
        $crate::program_entrypoint!($process_instruction, $maximum);
        $crate::default_allocator!();
        $crate::default_panic_handler!();
    };
}

/// Declare the program entrypoint.
///
/// This macro is similar to the [`crate::entrypoint!`] macro, but it does
/// not set up a global allocator nor a panic handler. This is useful when the program will set up
/// its own allocator and panic handler.
///
/// The first argument is the name of a function with this type signature:
///
/// ```ignore
/// fn process_instruction(
///     program_id: &Pubkey,      // Public key of the account the program was loaded into
///     accounts: &[AccountInfo], // All accounts required to process the instruction
///     instruction_data: &[u8],  // Serialized instruction-specific data
/// ) -> ProgramResult;
/// ```
/// The argument is defined as an `expr`, which allows the use of any function pointer not just
/// identifiers in the current scope.
///
/// There is a second optional argument that allows to specify the maximum number of accounts
/// expected by instructions of the program. This is useful to reduce the stack size requirement
/// for the entrypoint, as the default is set to [`crate::MAX_TX_ACCOUNTS`]. If the program
/// receives more accounts than the specified maximum, these accounts will be ignored.
#[macro_export]
macro_rules! program_entrypoint {
    ( $process_instruction:expr ) => {
        $crate::program_entrypoint!($process_instruction, { $crate::MAX_TX_ACCOUNTS });
    };
    ( $process_instruction:expr, $maximum:expr ) => {
        /// Program entrypoint.
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            const UNINIT: core::mem::MaybeUninit<$crate::account_info::AccountInfo> =
                core::mem::MaybeUninit::<$crate::account_info::AccountInfo>::uninit();
            // Create an array of uninitialized account infos.
            let mut accounts = [UNINIT; $maximum];

            let (program_id, count, instruction_data) =
                $crate::entrypoint::deserialize::<$maximum>(input, &mut accounts);

            // Call the program's entrypoint passing `count` account infos; we know that
            // they are initialized so we cast the pointer to a slice of `[AccountInfo]`.
            match $process_instruction(
                &program_id,
                core::slice::from_raw_parts(accounts.as_ptr() as _, count),
                &instruction_data,
            ) {
                Ok(()) => $crate::SUCCESS,
                Err(error) => error.into(),
            }
        }
    };
}

/// Align a pointer to the BPF alignment of `u128`.
macro_rules! align_pointer {
    ($ptr:ident) => {
        // integer-to-pointer cast: the resulting pointer will have the same provenance as
        // the original pointer and it follows the alignment requirement for the input.
        (($ptr as usize + (BPF_ALIGN_OF_U128 - 1)) & !(BPF_ALIGN_OF_U128 - 1)) as *mut u8
    };
}

/// A macro to repeat a pattern to process an account `n` times,
/// where `n` is the number of `_` tokens in the input.
///
/// The main advantage of this macro is to inline the code to process
/// `n` accounts, which is useful to reduce the number of jumps required.
/// As a result, it reduces the number of CUs required to process each
/// account.
///
/// Note that this macro emits code to update both the `input` and
/// `accounts` pointers.
macro_rules! process_n_accounts {
    // Base case: no tokens left.
    ( () => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {};

    // Recursive case: one `_` token per repetition.
    ( ( _ $($rest:tt)* ) => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {
        process_n_accounts!(@process_account => ($input, $accounts, $accounts_slice));
        process_n_accounts!(($($rest)*) => ($input, $accounts, $accounts_slice));
    };

    // Process one account.
    ( @process_account => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {
        // Increment the `accounts` pointer to the next account.
        $accounts = $accounts.add(1);

        // Read the next account.
        let account: *mut Account = $input as *mut Account;
        // Adds an 8-bytes offset for:
        //   - rent epoch in case of a non-duplicated account
        //   - duplicated marker + 7 bytes of padding in case of a duplicated account
        $input = $input.add(size_of::<u64>());

        if (*account).borrow_state != NON_DUP_MARKER {
            clone_account_info($accounts, $accounts_slice, (*account).borrow_state);
        } else {
            $accounts.write(AccountInfo { raw: account });

            $input = $input.add(STATIC_ACCOUNT_DATA);
            $input = $input.add((*account).data_len as usize);
            $input = align_pointer!($input);
        }
    };
}

/// Convenience macro to transform the number of accounts to process
/// into a pattern of `_` tokens for the [`process_n_accounts`] macro.
macro_rules! process_accounts {
    ( 1 => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {
        process_n_accounts!( (_) => ( $input, $accounts, $accounts_slice ));
    };
    ( 2 => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {
        process_n_accounts!( (_ _) => ( $input, $accounts, $accounts_slice ));
    };
    ( 3 => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {
        process_n_accounts!( (_ _ _) => ( $input, $accounts, $accounts_slice ));
    };
    ( 4 => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {
        process_n_accounts!( (_ _ _ _) => ( $input, $accounts, $accounts_slice ));
    };
    ( 5 => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {
        process_n_accounts!( (_ _ _ _ _) => ( $input, $accounts, $accounts_slice ));
    };
}

/// Create an `AccountInfo` referencing the same account referenced
/// by the `AccountInfo` at the specified `index`.
///
/// # Safety
///
/// The caller must ensure that:
///   - `accounts` pointer must point to an array of `AccountInfo`s where
///      the new `AccountInfo` will be written.
///   - `accounts_slice` pointer must point to a slice of `AccountInfo`s
///     already initialized.
///   - `index` is a valid index in the `accounts_slice`.
//
// Note: The function is marked as `cold` to stop the compiler from optimizing the
// parsing of duplicated accounts, which leads to an overall increase in CU
// consumption.
#[cold]
#[inline(always)]
unsafe fn clone_account_info(
    accounts: *mut AccountInfo,
    accounts_slice: *const AccountInfo,
    index: u8,
) {
    accounts.write(AccountInfo {
        raw: (*accounts_slice.add(index as usize)).raw,
    });
}

/// Parse the arguments from the runtime input buffer.
///
/// This function parses the `accounts`, `instruction_data` and `program_id` from
/// the input buffer. The `MAX_ACCOUNTS` constant defines the maximum number of accounts
/// that can be parsed from the input buffer. If the number of accounts in the input buffer
/// exceeds `MAX_ACCOUNTS`, the excess accounts will be skipped (ignored).
///
/// # Safety
///
/// The caller must ensure that the `input` buffer is valid, i.e., it represents the
/// program input parameters serialized by the SVM loader. Additionally, the `input`
/// should last for the lifetime of the program execution since the returned values
/// reference the `input`.
#[inline(always)]
pub unsafe fn deserialize<const MAX_ACCOUNTS: usize>(
    mut input: *mut u8,
    accounts: &mut [MaybeUninit<AccountInfo>; MAX_ACCOUNTS],
) -> (&'static Pubkey, usize, &'static [u8]) {
    // Ensure that MAX_ACCOUNTS is less than or equal to the maximum number of accounts
    // (MAX_TX_ACCOUNTS) that can be processed in a transaction.
    const {
        assert!(
            MAX_ACCOUNTS <= MAX_TX_ACCOUNTS,
            "MAX_ACCOUNTS must be less than or equal to MAX_TX_ACCOUNTS"
        );
    }

    // Number of accounts to process.
    let mut processed = *(input as *const u64) as usize;
    // Skip the number of accounts (8 bytes).
    input = input.add(size_of::<u64>());

    if processed > 0 {
        let mut accounts = accounts.as_mut_ptr() as *mut AccountInfo;
        // Represents the beginning of the accounts slice.
        let accounts_slice = accounts;

        // The first account is always non-duplicated, so process
        // it directly as such.
        let account: *mut Account = input as *mut Account;
        accounts.write(AccountInfo { raw: account });

        input = input.add(STATIC_ACCOUNT_DATA + size_of::<u64>());
        input = input.add((*account).data_len as usize);
        input = align_pointer!(input);

        if processed > 1 {
            // The number of accounts to process (`to_process_plus_one`) is limited to
            // `MAX_ACCOUNTS`, which is the capacity of the accounts array. When there
            // are more accounts to process than the maximum, we still need to skip the
            // remaining accounts (`to_skip`) to move the input pointer to the instruction
            // data. At the end, we return the number of accounts processed (`processed`),
            // which represents the accounts initialized in the `accounts` slice.
            //
            // Note that `to_process_plus_one` includes the first (already processed)
            // account to avoid decrementing the value. The actual number of remaining
            // accounts to process is `to_process_plus_one - 1`.
            let mut to_process_plus_one = if MAX_ACCOUNTS < MAX_TX_ACCOUNTS {
                min(processed, MAX_ACCOUNTS)
            } else {
                processed
            };

            let mut to_skip = processed - to_process_plus_one;
            processed = to_process_plus_one;

            // This is an optimization to reduce the number of jumps required
            // to process the accounts. The macro `process_accounts` will generate
            // inline code to process the specified number of accounts.
            if to_process_plus_one == 2 {
                process_accounts!(1 => (input, accounts, accounts_slice));
            } else {
                while to_process_plus_one > 5 {
                    // Process 5 accounts at a time.
                    process_accounts!(5 => (input, accounts, accounts_slice));
                    to_process_plus_one -= 5;
                }

                // There might be remaining accounts to process.
                match to_process_plus_one {
                    5 => {
                        process_accounts!(4 => (input, accounts, accounts_slice));
                    }
                    4 => {
                        process_accounts!(3 => (input, accounts, accounts_slice));
                    }
                    3 => {
                        process_accounts!(2 => (input, accounts, accounts_slice));
                    }
                    2 => {
                        process_accounts!(1 => (input, accounts, accounts_slice));
                    }
                    1 => (),
                    _ => {
                        // SAFETY: `while` loop above makes sure that `to_process_plus_one`
                        // has 1 to 5 entries left.
                        unsafe { core::hint::unreachable_unchecked() }
                    }
                }
            }

            // Process any remaining accounts to move the offset to the instruction
            // data (there is a duplication of logic but we avoid testing whether we
            // have space for the account or not).
            //
            // There might be accounts to skip only when `MAX_ACCOUNTS < MAX_TX_ACCOUNTS`
            // so this allows the compiler to optimize the code and avoid the loop when
            // `MAX_ACCOUNTS == MAX_TX_ACCOUNTS`.
            if MAX_ACCOUNTS < MAX_TX_ACCOUNTS {
                while to_skip > 0 {
                    // Marks the account as skipped.
                    to_skip -= 1;

                    // Read the next account.
                    let account: *mut Account = input as *mut Account;
                    // Adds an 8-bytes offset for:
                    //   - rent epoch in case of a non-duplicated account
                    //   - duplicated marker + 7 bytes of padding in case of a duplicated account
                    input = input.add(size_of::<u64>());

                    if (*account).borrow_state == NON_DUP_MARKER {
                        input = input.add(STATIC_ACCOUNT_DATA);
                        input = input.add((*account).data_len as usize);
                        input = align_pointer!(input);
                    }
                }
            }
        }
    }

    // instruction data
    let instruction_data_len = *(input as *const u64) as usize;
    input = input.add(size_of::<u64>());

    let instruction_data = { from_raw_parts(input, instruction_data_len) };
    let input = input.add(instruction_data_len);

    // program id
    let program_id: &Pubkey = &*(input as *const Pubkey);

    (program_id, processed, instruction_data)
}

/// Default panic hook.
///
/// This macro sets up a default panic hook that logs the panic message and the file where the
/// panic occurred. It acts as a hook after Rust runtime panics; syscall `abort()` will be called
/// after it returns.
///
/// Note that this requires the `"std"` feature to be enabled.
#[cfg(feature = "std")]
#[macro_export]
macro_rules! default_panic_handler {
    () => {
        /// Default panic handler.
        #[cfg(target_os = "solana")]
        #[no_mangle]
        fn custom_panic(info: &core::panic::PanicInfo<'_>) {
            // Panic reporting.
            $crate::msg!("{}", info);
        }
    };
}

/// Default panic hook.
///
/// This macro sets up a default panic hook that logs the file where the panic occurred. It acts
/// as a hook after Rust runtime panics; syscall `abort()` will be called after it returns.
///
/// This is used when the `"std"` feature is disabled, while either the program or any of its
/// dependencies are not `no_std`.
#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! default_panic_handler {
    () => {
        /// Default panic handler.
        #[cfg(target_os = "solana")]
        #[no_mangle]
        fn custom_panic(info: &core::panic::PanicInfo<'_>) {
            if let Some(location) = info.location() {
                $crate::log::sol_log(location.file());
            }
            // Panic reporting.
            $crate::log::sol_log("** PANICKED **");
        }
    };
}

/// A global `#[panic_handler]` for `no_std` programs.
///
/// This macro sets up a default panic handler that logs the location (file, line and column)
/// where the panic occurred and then calls the syscall `abort()`.
///
/// This macro can only be used when all crates are `no_std` and the `"std"` feature is
/// disabled.
#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! nostd_panic_handler {
    () => {
        /// A panic handler for `no_std`.
        #[cfg(target_os = "solana")]
        #[no_mangle]
        #[panic_handler]
        fn handler(info: &core::panic::PanicInfo<'_>) -> ! {
            if let Some(location) = info.location() {
                unsafe {
                    $crate::syscalls::sol_panic_(
                        location.file().as_ptr(),
                        location.file().len() as u64,
                        location.line() as u64,
                        location.column() as u64,
                    )
                }
            } else {
                // Panic reporting.
                $crate::log::sol_log("** PANICKED **");
                unsafe { $crate::syscalls::abort() }
            }
        }

        /// A panic handler for when the program is compiled on a target different than
        /// `"solana"`.
        ///
        /// This links the `std` library, which will set up a default panic handler.
        #[cfg(not(target_os = "solana"))]
        mod __private_panic_handler {
            extern crate std as __std;
        }
    };
}

/// Default global allocator.
///
/// This macro sets up a default global allocator that uses a bump allocator to allocate memory.
#[macro_export]
macro_rules! default_allocator {
    () => {
        #[cfg(target_os = "solana")]
        #[global_allocator]
        static A: $crate::entrypoint::BumpAllocator = $crate::entrypoint::BumpAllocator {
            start: $crate::entrypoint::HEAP_START_ADDRESS as usize,
            len: $crate::entrypoint::HEAP_LENGTH,
        };

        /// A default allocator for when the program is compiled on a target different than
        /// `"solana"`.
        ///
        /// This links the `std` library, which will set up a default global allocator.
        #[cfg(not(target_os = "solana"))]
        mod __private_alloc {
            extern crate std as __std;
        }
    };
}

/// A global allocator that does not allocate memory.
///
/// Using this macro with the "`std`" feature enabled will result in a compile error.
#[cfg(feature = "std")]
#[macro_export]
macro_rules! no_allocator {
    () => {
        compile_error!("Feature 'std' cannot be enabled.");
    };
}

/// A global allocator that does not dynamically allocate memory.
///
/// This macro sets up a global allocator that denies all dynamic allocations, while
/// allowing static ("manual") allocations. This is useful when the program does not need to
/// dynamically allocate memory and manages their own allocations.
///
/// The program will panic if it tries to dynamically allocate memory.
///
/// This is used when the `"std"` feature is disabled.
#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! no_allocator {
    () => {
        #[cfg(target_os = "solana")]
        #[global_allocator]
        static A: $crate::entrypoint::NoAllocator = $crate::entrypoint::NoAllocator;

        /// Allocates memory for the given type `T` at the specified offset in the
        /// heap reserved address space.
        ///
        /// # Safety
        ///
        /// It is the caller's responsibility to ensure that the offset does not
        /// overlap with previous allocations and that type `T` can hold the bit-pattern
        /// `0` as a valid value.
        ///
        /// For types that cannot hold the bit-pattern `0` as a valid value, use
        /// `core::mem::MaybeUninit<T>` to allocate memory for the type and
        /// initialize it later.
        //
        // Make this `const` once `const_mut_refs` is stable for the platform-tools
        // toolchain Rust version.
        #[inline(always)]
        pub unsafe fn allocate_unchecked<T: Sized>(offset: usize) -> &'static mut T {
            // SAFETY: The pointer is within a valid range and aligned to `T`.
            unsafe { &mut *(calculate_offset::<T>(offset) as *mut T) }
        }

        #[inline(always)]
        const fn calculate_offset<T: Sized>(offset: usize) -> usize {
            let start = $crate::entrypoint::HEAP_START_ADDRESS as usize + offset;
            let end = start + core::mem::size_of::<T>();

            // Assert if the allocation does not exceed the heap size.
            assert!(
                end <= $crate::entrypoint::HEAP_START_ADDRESS as usize
                    + $crate::entrypoint::HEAP_LENGTH,
                "allocation exceeds heap size"
            );

            // Assert if the pointer is aligned to `T`.
            assert!(
                start % core::mem::align_of::<T>() == 0,
                "offset is not aligned"
            );

            start
        }

        /// A default allocator for when the program is compiled on a target different than
        /// `"solana"`.
        ///
        /// This links the `std` library, which will set up a default global allocator.
        #[cfg(not(target_os = "solana"))]
        mod __private_alloc {
            extern crate std as __std;
        }
    };
}

#[cfg(target_os = "solana")]
mod alloc {
    //! The bump allocator used as the default rust heap when running programs.

    extern crate alloc;

    /// The bump allocator used as the default rust heap when running programs.
    pub struct BumpAllocator {
        pub start: usize,
        pub len: usize,
    }

    /// Integer arithmetic in this global allocator implementation is safe when
    /// operating on the prescribed [`HEAP_START_ADDRESS`] and [`HEAP_LENGTH`]. Any
    /// other use may overflow and is thus unsupported and at one's own risk.
    #[allow(clippy::arithmetic_side_effects)]
    unsafe impl alloc::alloc::GlobalAlloc for BumpAllocator {
        /// Allocates memory as a bump allocator.
        #[inline]
        unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
            let pos_ptr = self.start as *mut usize;

            let mut pos = *pos_ptr;
            if pos == 0 {
                // First time, set starting position.
                pos = self.start + self.len;
            }
            pos = pos.saturating_sub(layout.size());
            pos &= !(layout.align().wrapping_sub(1));
            if pos < self.start + core::mem::size_of::<*mut u8>() {
                return core::ptr::null_mut();
            }
            *pos_ptr = pos;
            pos as *mut u8
        }
        #[inline]
        unsafe fn dealloc(&self, _: *mut u8, _: core::alloc::Layout) {
            // I'm a bump allocator, I don't free.
        }
    }
}

#[cfg(not(feature = "std"))]
/// An allocator that does not allocate memory.
pub struct NoAllocator;

#[cfg(not(feature = "std"))]
unsafe impl core::alloc::GlobalAlloc for NoAllocator {
    #[inline]
    unsafe fn alloc(&self, _: core::alloc::Layout) -> *mut u8 {
        panic!("** NO ALLOCATOR **");
    }

    #[inline]
    unsafe fn dealloc(&self, _: *mut u8, _: core::alloc::Layout) {
        // I deny all allocations, so I don't need to free.
    }
}

#[cfg(all(test, not(target_os = "solana")))]
mod tests {
    extern crate std;

    use core::{alloc::Layout, ptr::copy_nonoverlapping};
    use std::{
        alloc::{alloc, dealloc},
        vec,
    };

    use super::*;

    /// The mock program ID used for testing.
    const MOCK_PROGRAM_ID: Pubkey = [5u8; 32];

    /// An uninitialized account info.
    const UNINIT: MaybeUninit<AccountInfo> = MaybeUninit::<AccountInfo>::uninit();

    /// Struct representing a memory region with a specific alignment.
    struct AlignedMemory {
        ptr: *mut u8,
        layout: Layout,
    }

    impl AlignedMemory {
        pub fn new(len: usize) -> Self {
            let layout = Layout::from_size_align(len, BPF_ALIGN_OF_U128).unwrap();
            // SAFETY: `align` is set to `BPF_ALIGN_OF_U128`.
            unsafe {
                let ptr = alloc(layout);
                if ptr.is_null() {
                    std::alloc::handle_alloc_error(layout);
                }
                AlignedMemory { ptr, layout }
            }
        }

        /// Write data to the memory region at the specified offset.
        ///
        /// # Safety
        ///
        /// The caller must ensure that the `data` length does not exceed the
        /// remaining space in the memory region starting from the `offset`.
        pub unsafe fn write(&mut self, data: &[u8], offset: usize) {
            copy_nonoverlapping(data.as_ptr(), self.ptr.add(offset), data.len());
        }

        /// Return a mutable pointer to the memory region.
        pub fn as_mut_ptr(&mut self) -> *mut u8 {
            self.ptr
        }
    }

    impl Drop for AlignedMemory {
        fn drop(&mut self) {
            unsafe {
                dealloc(self.ptr, self.layout);
            }
        }
    }

    /// Creates an input buffer with a specified number of accounts and
    /// instruction data.
    ///
    /// This function mimics the input buffer created by the SVM loader.
    /// Each account created has zeroed data, apart from the `data_len`
    /// field, which is set to the index of the account.
    ///
    /// # Safety
    ///
    /// The returned `AlignedMemory` should only be used within the test
    /// context.
    unsafe fn create_input(accounts: usize, instruction_data: &[u8]) -> AlignedMemory {
        let mut input = AlignedMemory::new(1_000_000_000);
        // Number of accounts.
        input.write(&(accounts as u64).to_le_bytes(), 0);
        let mut offset = size_of::<u64>();

        for i in 0..accounts {
            // Account data.
            let mut account = [0u8; STATIC_ACCOUNT_DATA + size_of::<u64>()];
            account[0] = NON_DUP_MARKER;
            // Set the accounts data length. The actual account data is zeroed.
            account[80..88].copy_from_slice(&i.to_le_bytes());
            input.write(&account, offset);
            offset += account.len();
            // Padding for the account data to align to `BPF_ALIGN_OF_U128`.
            let padding_for_data = (i + (BPF_ALIGN_OF_U128 - 1)) & !(BPF_ALIGN_OF_U128 - 1);
            input.write(&vec![0u8; padding_for_data], offset);
            offset += padding_for_data;
        }

        // Instruction data length.
        input.write(&instruction_data.len().to_le_bytes(), offset);
        offset += size_of::<u64>();
        // Instruction data.
        input.write(instruction_data, offset);
        offset += instruction_data.len();
        // Program ID (mock).
        input.write(&MOCK_PROGRAM_ID, offset);

        input
    }

    /// Creates an input buffer with a specified number of accounts, including
    /// duplicated accounts, and instruction data.
    ///
    /// This function differs from `create_input` in that it creates accounts
    /// with a marker indicating that they are duplicated. There will be
    /// `accounts - duplicated` unique accounts, and the remaining `duplicated`
    /// accounts will be duplicates of the last unique account.
    ///
    /// This function mimics the input buffer created by the SVM loader.
    /// Each account created has zeroed data, apart from the `data_len`
    /// field, which is set to the index of the account.
    ///
    /// # Safety
    ///
    /// The returned `AlignedMemory` should only be used within the test
    /// context.
    unsafe fn create_input_with_duplicates(
        accounts: usize,
        instruction_data: &[u8],
        duplicated: usize,
    ) -> AlignedMemory {
        let mut input = AlignedMemory::new(1_000_000_000);
        // Number of accounts.
        input.write(&(accounts as u64).to_le_bytes(), 0);
        let mut offset = size_of::<u64>();

        if accounts > 0 {
            assert!(
                duplicated < accounts,
                "Duplicated accounts must be less than total accounts"
            );
            let unique = accounts - duplicated;

            for i in 0..unique {
                // Account data.
                let mut account = [0u8; STATIC_ACCOUNT_DATA + size_of::<u64>()];
                account[0] = NON_DUP_MARKER;
                // Set the accounts data length. The actual account data is zeroed.
                account[80..88].copy_from_slice(&i.to_le_bytes());
                input.write(&account, offset);
                offset += account.len();
                // Padding for the account data to align to `BPF_ALIGN_OF_U128`.
                let padding_for_data = (i + (BPF_ALIGN_OF_U128 - 1)) & !(BPF_ALIGN_OF_U128 - 1);
                input.write(&vec![0u8; padding_for_data], offset);
                offset += padding_for_data;
            }

            // Remaining accounts are duplicated of the last unique account.
            for _ in unique..accounts {
                input.write(&[(unique - 1) as u8, 0, 0, 0, 0, 0, 0, 0], offset);
                offset += size_of::<u64>();
            }
        }

        // Instruction data length.
        input.write(&instruction_data.len().to_le_bytes(), offset);
        offset += size_of::<u64>();
        // Instruction data.
        input.write(instruction_data, offset);
        offset += instruction_data.len();
        // Program ID (mock).
        input.write(&MOCK_PROGRAM_ID, offset);

        input
    }

    /// Asserts that the accounts slice contains the expected number of accounts
    /// and that each account's data length matches its index.
    fn assert_accounts(accounts: &[MaybeUninit<AccountInfo>]) {
        for (i, account) in accounts.iter().enumerate() {
            let account_info = unsafe { account.assume_init_ref() };
            assert_eq!(account_info.data_len(), i);
        }
    }

    /// Asserts that the accounts slice contains the expected number of accounts
    /// and all accounts are duplicated, apart from the first one.
    fn assert_duplicated_accounts(accounts: &[MaybeUninit<AccountInfo>], duplicated: usize) {
        assert!(accounts.len() > duplicated);

        let unique = accounts.len() - duplicated;

        // Unique accounts should have `data_len` equal to their index.
        for (i, account) in accounts[..unique].iter().enumerate() {
            let account_info = unsafe { account.assume_init_ref() };
            assert_eq!(account_info.data_len(), i);
        }

        // Duplicated accounts should have the same `data_len` as the
        // last unique account.
        for account in accounts[unique..].iter() {
            let account_info = unsafe { account.assume_init_ref() };
            let duplicated = unsafe { accounts[unique - 1].assume_init_ref() };

            assert_eq!(account_info.raw, duplicated.raw);
            assert_eq!(account_info.data_len(), duplicated.data_len());
        }
    }

    #[test]
    fn test_deserialize() {
        let ix_data = [3u8; 100];

        // Input with 0 accounts.

        let mut input = unsafe { create_input(0, &ix_data) };
        let mut accounts = [UNINIT; 1];

        let (program_id, count, parsed_ix_data) =
            unsafe { deserialize(input.as_mut_ptr(), &mut accounts) };

        assert_eq!(count, 0);
        assert_eq!(program_id, &MOCK_PROGRAM_ID);
        assert_eq!(&ix_data, parsed_ix_data);

        // Input with 3 accounts but the accounts array has only space
        // for 1.

        let mut input = unsafe { create_input(3, &ix_data) };
        let mut accounts = [UNINIT; 1];

        let (program_id, count, parsed_ix_data) =
            unsafe { deserialize(input.as_mut_ptr(), &mut accounts) };

        assert_eq!(count, 1);
        assert_eq!(program_id, &MOCK_PROGRAM_ID);
        assert_eq!(&ix_data, parsed_ix_data);
        assert_accounts(&accounts[..count]);

        // Input with `MAX_TX_ACCOUNTS` accounts but accounts array has
        // only space for 64.

        let mut input = unsafe { create_input(MAX_TX_ACCOUNTS, &ix_data) };
        let mut accounts = [UNINIT; 64];

        let (program_id, count, parsed_ix_data) =
            unsafe { deserialize(input.as_mut_ptr(), &mut accounts) };

        assert_eq!(count, 64);
        assert_eq!(program_id, &MOCK_PROGRAM_ID);
        assert_eq!(&ix_data, parsed_ix_data);
        assert_accounts(&accounts);
    }

    #[test]
    fn test_deserialize_duplicated() {
        let ix_data = [3u8; 100];

        // Input with 0 accounts.

        let mut input = unsafe { create_input_with_duplicates(0, &ix_data, 0) };
        let mut accounts = [UNINIT; 1];

        let (program_id, count, parsed_ix_data) =
            unsafe { deserialize(input.as_mut_ptr(), &mut accounts) };

        assert_eq!(count, 0);
        assert_eq!(program_id, &MOCK_PROGRAM_ID);
        assert_eq!(&ix_data, parsed_ix_data);

        // Input with 3 (1 + 2 duplicated) accounts but the accounts array has only
        // space for 2. The assert checks that the second account is a duplicate of
        // the first one and the first one is unique.

        let mut input = unsafe { create_input_with_duplicates(3, &ix_data, 2) };
        let mut accounts = [UNINIT; 2];

        let (program_id, count, parsed_ix_data) =
            unsafe { deserialize(input.as_mut_ptr(), &mut accounts) };

        assert_eq!(count, 2);
        assert_eq!(program_id, &MOCK_PROGRAM_ID);
        assert_eq!(&ix_data, parsed_ix_data);
        assert_duplicated_accounts(&accounts[..count], 1);

        // Input with `MAX_TX_ACCOUNTS` accounts (only 32 unique ones) but accounts
        // array has only space for 64. The assert checks that the first 32 accounts
        // are unique and the rest are duplicates of the account at index 31.

        let mut input = unsafe {
            create_input_with_duplicates(MAX_TX_ACCOUNTS, &ix_data, MAX_TX_ACCOUNTS - 32)
        };
        let mut accounts = [UNINIT; 64];

        let (program_id, count, parsed_ix_data) =
            unsafe { deserialize(input.as_mut_ptr(), &mut accounts) };

        assert_eq!(count, 64);
        assert_eq!(program_id, &MOCK_PROGRAM_ID);
        assert_eq!(&ix_data, parsed_ix_data);
        assert_duplicated_accounts(&accounts, 32);
    }
}
