//! Macros and functions for defining the program entrypoint and setting up
//! global handlers.

pub mod lazy;
pub use lazy::{InstructionContext, MaybeAccount};

#[cfg(target_os = "solana")]
pub use alloc::BumpAllocator;

use crate::{
    account_info::{Account, AccountInfo, MAX_PERMITTED_DATA_INCREASE},
    pubkey::Pubkey,
    BPF_ALIGN_OF_U128, NON_DUP_MARKER,
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

/// Declare the program entrypoint and set up global handlers.
///
/// This macro is deprecated and will be removed in a future version. It is recommended to
/// use the [`entrypoint_with_allocator_and_panic_handler`] macro instead.
#[deprecated(
    since = "0.8.4",
    note = "Use the `entrypoint_with_allocator_and_panic_handler` macro instead"
)]
#[macro_export]
macro_rules! entrypoint {
    ( $process_instruction:ident ) => {
        entrypoint!($process_instruction, { $crate::MAX_TX_ACCOUNTS });
    };
    ( $process_instruction:ident, $maximum:expr ) => {
        $crate::entrypoint_with_allocator_and_panic_handler!($process_instruction, $maximum);
    };
}

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
/// It also sets up a [global allocator] and [panic handler], using the [`crate::default_allocator`]
/// and [`crate::default_panic_handler`] macros.
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
///
/// The second (optional) argument is the maximum number of accounts that the program is expecting.
/// A program can receive more than the specified maximum, but any account exceeding the maximum will
/// be ignored. When the maximum is not specified, the default is `64`. This is currently the [maximum
/// number of accounts] that a transaction may lock in a block.
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
///         entrypoint_with_allocator_and_panic_handler,
///         msg,
///         pubkey::Pubkey,
///         ProgramResult
///     };
///
///     entrypoint_with_allocator_and_panic_handler!(process_instruction);
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
/// linker or not. The `entrypoint_with_allocator_and_panic_handler` macro will set up a default
/// panic "hook", that works with the `#[panic_handler]` set by the `std`. Therefore, this macro
/// should be used when the program or any of its dependencies are dependent on the `std` library.
///
/// When the program and all its dependencies are `no_std`, it is necessary to set a
/// `#[panic_handler]` to handle panics. This is done by the [`nostd_panic_handler`] macro. In this
/// case, it is not possible to use the `entrypoint_with_allocator_and_panic_handler` macro. Use the
/// [`program_entrypoint`] macro instead and set up the allocator and panic handler manually.
#[macro_export]
macro_rules! entrypoint_with_allocator_and_panic_handler {
    ( $process_instruction:ident ) => {
        entrypoint_with_allocator_and_panic_handler!($process_instruction, {
            $crate::MAX_TX_ACCOUNTS
        });
    };
    ( $process_instruction:ident, $maximum:expr ) => {
        $crate::program_entrypoint!($process_instruction, $maximum);
        $crate::default_allocator!();
        $crate::default_panic_handler!();
    };
}

/// Declare the program entrypoint.
///
/// This macro is similar to the [`entrypoint_with_allocator_and_panic_handler`] macro, but it does
/// not set up a global allocator nor a panic handler. This is useful when the program will set up
/// its own allocator and panic handler.
#[macro_export]
macro_rules! program_entrypoint {
    ( $process_instruction:ident ) => {
        program_entrypoint!($process_instruction, { $crate::MAX_TX_ACCOUNTS });
    };
    ( $process_instruction:ident, $maximum:expr ) => {
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

/// Deserialize the input arguments.
///
/// This can only be called from the entrypoint function of a Solana program and with
/// a buffer that was serialized by the runtime.
#[allow(clippy::cast_ptr_alignment, clippy::missing_safety_doc)]
#[inline(always)]
pub unsafe fn deserialize<'a, const MAX_ACCOUNTS: usize>(
    input: *mut u8,
    accounts: &mut [core::mem::MaybeUninit<AccountInfo>],
) -> (&'a Pubkey, usize, &'a [u8]) {
    let mut offset: usize = 0;

    // total number of accounts present; it only process up to MAX_ACCOUNTS
    let total_accounts = *(input.add(offset) as *const u64) as usize;
    offset += core::mem::size_of::<u64>();

    let processed = if total_accounts > 0 {
        // number of accounts to process (limited to MAX_ACCOUNTS)
        let processed = core::cmp::min(total_accounts, MAX_ACCOUNTS);

        for i in 0..processed {
            let account_info: *mut Account = input.add(offset) as *mut _;

            if (*account_info).borrow_state == NON_DUP_MARKER {
                // repurpose the borrow state to track borrows
                (*account_info).borrow_state = 0b_0000_0000;

                offset += core::mem::size_of::<Account>();
                offset += (*account_info).data_len as usize;
                offset += MAX_PERMITTED_DATA_INCREASE;
                offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128);
                offset += core::mem::size_of::<u64>();

                accounts[i].write(AccountInfo { raw: account_info });
            } else {
                offset += core::mem::size_of::<u64>();
                // duplicated account – clone the original pointer using `borrow_state` since it represents the
                // index of the duplicated account passed by the runtime.
                accounts[i].write(
                    accounts
                        .get_unchecked((*account_info).borrow_state as usize)
                        .assume_init_ref()
                        .clone(),
                );
            }
        }

        // process any remaining accounts to move the offset to the instruction
        // data (there is a duplication of logic but we avoid testing whether we
        // have space for the account or not)
        for _ in processed..total_accounts {
            let account_info: *mut Account = input.add(offset) as *mut _;

            if (*account_info).borrow_state == NON_DUP_MARKER {
                offset += core::mem::size_of::<Account>();
                offset += (*account_info).data_len as usize;
                offset += MAX_PERMITTED_DATA_INCREASE;
                offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128);
                offset += core::mem::size_of::<u64>();
            } else {
                offset += core::mem::size_of::<u64>();
            }
        }

        processed
    } else {
        // no accounts to process
        0
    };

    // instruction data
    let instruction_data_len = *(input.add(offset) as *const u64) as usize;
    offset += core::mem::size_of::<u64>();

    let instruction_data = { core::slice::from_raw_parts(input.add(offset), instruction_data_len) };
    offset += instruction_data_len;

    // program id
    let program_id: &Pubkey = &*(input.add(offset) as *const Pubkey);

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

/// A global allocator that does not allocate memory.
///
/// This macro sets up a global allocator that denies all allocations. This is useful
/// when the program does not need to allocate memory. The program will panic if it tries
/// to allocate memory.
#[deprecated(since = "0.8.4", note = "Use the `static_allocator` macro instead")]
#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! no_allocator {
    () => {
        $crate::static_allocator!();
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
macro_rules! static_allocator {
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
