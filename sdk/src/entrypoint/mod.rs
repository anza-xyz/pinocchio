//! Macros and functions for defining the program entrypoint and setting up
//! global handlers.
//!
//! When an instruction is directed at an executable program, the loader
//! configures the program's execution environment, serializes the program's
//! input parameters, invokes the program's entrypoint, and reports any errors
//! encountered. The input parameters are serialized into a byte array and
//! passed to the program's entrypoint. Each program is responsible for
//! deserializing these parameters on-chain.
//!
//! The input parameters are serialized as follows (all encoding is little
//! endian):
//!
//!```text
//! ┌─ 8 bytes unsigned (u64): number of accounts
//! │
//! ├─ For each account:
//! |   |
//! │   ├─ 1 byte: indicating if this is a duplicate account, if not a duplicate then
//! │   │          the value is 0xFF, otherwise the value is the index of the account
//! │   │          it is a duplicate of.
//! |   │
//! │   ├─ If the account is a duplicate:
//! |   |     |
//! │   │     └─ 7 bytes of padding
//! |   │
//! │   └─ If the account is not a duplicate:
//! |         |
//! │         ├─ 1 byte boolean, true if account is a signer
//! |         |
//! │         ├─ 1 byte boolean, true if account is writable
//! |         |
//! |         ├─ 1 byte boolean, true if account is executable
//! |         |
//! │         ├─ 4 bytes of padding (account data length stored here with `account-resize` feature)
//! |         |
//! │         ├─ 32 bytes: address of the account
//! |         |
//! │         ├─ 32 bytes: address of the program account owner
//! |         |
//! │         ├─ 8 bytes unsigned (u64): lamports held by the account
//! |         |
//! │         ├─ 8 bytes unsigned (u64): number of bytes of account data
//! |         |
//! │         ├─ <variable> bytes of account data
//! |         |
//! │         ├─ 10240 bytes of padding (used for resize)
//! |         |
//! │         ├─ <variable> bytes to align the offset to 8 bytes
//! |         |
//! │         └─ 8 bytes unsigned (u64): rent epoch of the account (not used)
//! │
//! ├─ 8 bytes unsigned (u64): number of bytes of instruction data
//! │
//! ├─ <variable> bytes of instruction data
//! │
//! ├─ 32 bytes: address of the program account
//! │
//! ├─ <variable> bytes to align the offset to 8 bytes
//! │
//! └─ <variable> bytes of account pointers (8 bytes x number of accounts), where each pointer
//!    points to the start of the corresponding account in the input buffer.
//! ```

/// Align a pointer to the BPF alignment of [`u128`].
macro_rules! align_pointer {
    ($ptr:expr) => {
        // Integer-to-pointer cast: first compute the aligned address as a `usize`,
        // since this is more CU-efficient than using `ptr::align_offset()` or the
        // strict provenance API (e.g., `ptr::with_addr()`). Then cast the result
        // back to a pointer. The resulting pointer is guaranteed to be valid
        // because it follows the layout serialized by the runtime.
        with_exposed_provenance_mut(
            ($ptr.expose_provenance() + (BPF_ALIGN_OF_U128 - 1)) & !(BPF_ALIGN_OF_U128 - 1),
        )
    };
}

pub mod lazy;

#[cfg(feature = "alloc")]
pub use alloc::BumpAllocator;
pub use lazy::{InstructionContext, MaybeAccount};
use {
    crate::{
        account::{AccountView, RuntimeAccount, MAX_PERMITTED_DATA_INCREASE},
        error::ProgramError,
        Address, ProgramResult, BPF_ALIGN_OF_U128, SUCCESS,
    },
    core::{
        alloc::{GlobalAlloc, Layout},
        mem::size_of,
        ptr::with_exposed_provenance_mut,
        slice::{from_raw_parts, from_raw_parts_mut},
    },
};

/// Start address of the memory region used for program heap.
pub const HEAP_START_ADDRESS: u64 = 0x300000000;

/// Length of the heap memory region used for program heap.
#[deprecated(since = "0.10.0", note = "Use `MAX_HEAP_LENGTH` instead")]
pub const HEAP_LENGTH: usize = 32 * 1024;

/// Maximum heap length in bytes that a program can request.
pub const MAX_HEAP_LENGTH: u32 = 256 * 1024;

/// Value used to indicate that a serialized account is not a duplicate.
pub const NON_DUP_MARKER: u8 = u8::MAX;

/// The "static" size of an account in the input buffer.
///
/// This is the size of the account header plus the maximum permitted data
/// increase.
const STATIC_ACCOUNT_DATA: usize = size_of::<RuntimeAccount>() + MAX_PERMITTED_DATA_INCREASE;

/// Declare the program entrypoint and set up global handlers.
///
/// The main difference from the standard (SDK) [`entrypoint`] macro is that
/// this macro represents an entrypoint that does not perform allocations or
/// copies when reading the input buffer.
///
/// [`entrypoint`]: https://docs.rs/solana-program-entrypoint/latest/solana_program_entrypoint/macro.entrypoint.html
///
/// This macro emits the common boilerplate necessary to begin program
/// execution, calling a provided function to process the program instruction
/// supplied by the runtime, and reporting its result to the runtime.
///
/// It also sets up a [global allocator] and [custom panic hook], using the
/// [`crate::default_allocator!`] and [`crate::default_panic_handler!`] macros.
///
/// The macro argument is the name of a function with this type signature:
///
/// ```ignore
/// fn process_instruction(
///     program_id: &Address,         // Address of the account the program was loaded into
///     accounts: &mut [AccountView], // All accounts required to process the instruction
///     instruction_data: &[u8],      // Serialized instruction-specific data
/// ) -> ProgramResult;
/// ```
/// The argument is defined as an `expr`, which allows the use of any function
/// pointer not just identifiers in the current scope.
///
/// [global allocator]: https://doc.rust-lang.org/stable/alloc/alloc/trait.GlobalAlloc.html
/// [custom panic hook]: https://github.com/anza-xyz/rust/blob/2830febbc59d44bdd7ad2c3b81731f1d08b96eba/library/std/src/sys/pal/sbf/mod.rs#L49
///
/// # Examples
///
/// Defining an entrypoint conditional on the `bpf-entrypoint` feature. Although
/// the `entrypoint` module is written inline in this example, it is common to
/// put it into its own file.
///
/// ```no_run
/// #[cfg(feature = "bpf-entrypoint")]
/// pub mod entrypoint {
///
///     use pinocchio::{
///         AccountView,
///         entrypoint,
///         Address,
///         ProgramResult
///     };
///
///     entrypoint!(process_instruction);
///
///     pub fn process_instruction(
///         program_id: &Address,
///         accounts: &mut [AccountView],
///         instruction_data: &[u8],
///     ) -> ProgramResult {
///         Ok(())
///     }
///
/// }
/// ```
///
/// # Important
///
/// The panic handler set up is different depending on whether the `std` library
/// is available to the linker or not. The `entrypoint` macro will set up a
/// default panic "hook", that works with the `#[panic_handler]` set by the
/// `std`. Therefore, this macro should be used when the program or any of its
/// dependencies are dependent on the `std` library.
///
/// When the program and all its dependencies are `no_std`, it is necessary to
/// set a `#[panic_handler]` to handle panics. This is done by the
/// [`crate::nostd_panic_handler`] macro. In this case, it is not possible to
/// use the `entrypoint` macro. Use the [`crate::program_entrypoint!`] macro
/// instead and set up the allocator and panic handler manually.
///
/// The compiler may inline the instruction handler (and its call tree) into the
/// generated `entrypoint`, which can significantly increase the entrypoint
/// stack frame. If your program has large instruction dispatch logic or builds
/// sizable CPI account arrays, consider adding `#[inline(never)]` to the
/// instruction handler to keep it out of the entrypoint stack frame and avoid
/// BPF stack overflows.
#[cfg(feature = "alloc")]
#[macro_export]
macro_rules! entrypoint {
    ( $process_instruction:expr ) => {
        $crate::program_entrypoint!($process_instruction);
        $crate::default_allocator!();
        $crate::default_panic_handler!();
    };
}

/// Declare the program entrypoint.
///
/// This macro is similar to the [`crate::entrypoint!`] macro, but it does not
/// set up a global allocator nor a panic handler. This is useful when the
/// program will set up its own allocator and panic handler.
///
/// The macro argument is the name of a function with this type signature:
///
/// ```ignore
/// fn process_instruction(
///     program_id: &Address,         // Address of the account the program was loaded into
///     accounts: &mut [AccountView], // All accounts required to process the instruction
///     instruction_data: &[u8],      // Serialized instruction-specific data
/// ) -> ProgramResult;
/// ```
/// The argument is defined as an `expr`, which allows the use of any function
/// pointer not just identifiers in the current scope.
#[macro_export]
macro_rules! program_entrypoint {
    ( $process_instruction:expr ) => {
        /// Program entrypoint.
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(
            program_input: *mut u8,
            instruction_data: *mut u8,
        ) -> u64 {
            $crate::entrypoint::process_entrypoint(
                program_input,
                instruction_data,
                $process_instruction,
            )
        }
    };
}

/// Convert a `ProgramError` into a `u64`.
///
/// This function is marked as `#[cold]` to move the error conversion from the
/// "hot path" of the entrypoint.
#[cold]
#[inline(never)]
fn program_error_to_u64(error: ProgramError) -> u64 {
    error.into()
}

/// Entrypoint deserialization.
///
/// This function inlines entrypoint deserialization for use in the
/// `program_entrypoint!` macro.
///
/// # Safety
///
/// The caller must ensure that both `program_input` and `instruction_data`
/// pointers are valid, i.e., they represent the input parameters serialized by
/// the SVM loader and their lifetimes last for the duration of the program
/// execution.
#[inline(always)]
pub unsafe fn process_entrypoint<F>(
    program_input: *mut u8,
    instruction_data: *mut u8,
    process_instruction: F,
) -> u64
where
    F: FnOnce(&Address, &mut [AccountView], &[u8]) -> ProgramResult,
{
    // Loads the instruction data length (8-bytes before the instruction data).
    let ix_data_len = *(instruction_data.sub(size_of::<u64>()) as *const u64) as usize;
    // Loads the `program_id`.
    let program_id = &*(instruction_data.add(ix_data_len) as *const Address);

    // The slice of account pointers is located right after the `program_id` +
    // alignment padding. The length of the slice is determined by the first
    // 8-bytes of `program_input`.

    let slice_ptr = instruction_data.add(ix_data_len + size_of::<Address>());

    let accounts = from_raw_parts_mut(
        align_pointer!(slice_ptr),
        *(program_input as *const u64) as usize,
    );

    // The instruction data slice is given by the `instruction_data` pointer.
    let instruction_data = from_raw_parts(instruction_data, ix_data_len);

    match process_instruction(program_id, accounts, instruction_data) {
        Ok(_) => SUCCESS,
        Err(e) => program_error_to_u64(e),
    }
}

/// Default panic hook.
///
/// This macro sets up a default panic hook that logs the file where the panic
/// occurred. It acts as a hook after Rust runtime panics; syscall `abort()`
/// will be called after it returns.
#[macro_export]
macro_rules! default_panic_handler {
    () => {
        /// Default panic handler.
        #[cfg(any(target_os = "solana", target_arch = "bpf"))]
        #[no_mangle]
        fn custom_panic(info: &core::panic::PanicInfo<'_>) {
            if let Some(location) = info.location() {
                let location = location.file();
                unsafe { $crate::syscalls::sol_log_(location.as_ptr(), location.len() as u64) };
            }
            // Panic reporting.
            const PANICKED: &str = "** PANICKED **";
            unsafe { $crate::syscalls::sol_log_(PANICKED.as_ptr(), PANICKED.len() as u64) };
        }
    };
}

/// A global `#[panic_handler]` for `no_std` programs.
///
/// This macro sets up a default panic handler that logs the location (file,
/// line and column) where the panic occurred and then calls the syscall
/// `abort()`.
///
/// This macro should be used when all crates are `no_std`.
#[macro_export]
macro_rules! nostd_panic_handler {
    () => {
        /// A panic handler for `no_std`.
        #[cfg(any(target_os = "solana", target_arch = "bpf"))]
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
                const PANICKED: &str = "** PANICKED **";
                unsafe {
                    $crate::syscalls::sol_log_(PANICKED.as_ptr(), PANICKED.len() as u64);
                    $crate::syscalls::abort();
                }
            }
        }

        /// A panic handler for when the program is compiled on a target different than
        /// `"solana"`.
        ///
        /// This links the `std` library, which will set up a default panic handler.
        #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
        mod __private_panic_handler {
            extern crate std as __std;
        }
    };
}

/// Default global allocator.
///
/// This macro sets up a default global allocator that uses a bump allocator to
/// allocate memory.
#[cfg(feature = "alloc")]
#[macro_export]
macro_rules! default_allocator {
    () => {
        #[cfg(any(target_os = "solana", target_arch = "bpf"))]
        #[global_allocator]
        static A: $crate::entrypoint::BumpAllocator = unsafe {
            $crate::entrypoint::BumpAllocator::new_unchecked(
                $crate::entrypoint::HEAP_START_ADDRESS as usize,
                // Use the maximum heap length allowed. Programs can request heap sizes up
                // to this value using the `ComputeBudget`.
                $crate::entrypoint::MAX_HEAP_LENGTH as usize,
            )
        };

        /// A default allocator for when the program is compiled on a target different
        /// than `"solana"`.
        ///
        /// This links the `std` library, which will set up a default global allocator.
        #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
        mod __private_alloc {
            extern crate std as __std;
        }
    };
}

/// A global allocator that does not dynamically allocate memory.
///
/// This macro sets up a global allocator that denies all dynamic allocations,
/// while allowing static ("manual") allocations. This is useful when the
/// program does not need to dynamically allocate memory and manages its own
/// allocations.
///
/// The program will panic if it tries to dynamically allocate memory.
///
/// This is used when the `"alloc"` feature is disabled.
#[macro_export]
macro_rules! no_allocator {
    () => {
        #[cfg(any(target_os = "solana", target_arch = "bpf"))]
        #[global_allocator]
        static A: $crate::entrypoint::NoAllocator = $crate::entrypoint::NoAllocator;

        /// Allocates memory for the given type `T` at the specified offset in the heap
        /// reserved address space.
        ///
        /// # Safety
        ///
        /// It is the caller's responsibility to ensure that the offset does not overlap
        /// with previous allocations and that type `T` can hold the bit-pattern `0` as
        /// a valid value.
        ///
        /// For types that cannot hold the bit-pattern `0` as a valid value, use
        /// [`core::mem::MaybeUninit<T>`] to allocate memory for the type and initialize
        /// it later.
        //
        // Make this `const` once `const_mut_refs` is stable for the platform-tools toolchain Rust
        // version.
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
                    + $crate::entrypoint::MAX_HEAP_LENGTH as usize,
                "allocation exceeds heap size"
            );

            // Assert if the pointer is aligned to `T`.
            assert!(
                start % core::mem::align_of::<T>() == 0,
                "offset is not aligned"
            );

            start
        }

        /// A default allocator for when the program is compiled on a target different
        /// than `"solana"`.
        ///
        /// This links the `std` library, which will set up a default global allocator.
        #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
        mod __private_alloc {
            extern crate std as __std;
        }
    };
}

/// An allocator that does not allocate memory.
#[cfg_attr(feature = "copy", derive(Copy))]
#[derive(Clone, Debug)]
pub struct NoAllocator;

unsafe impl GlobalAlloc for NoAllocator {
    #[inline]
    unsafe fn alloc(&self, _: Layout) -> *mut u8 {
        panic!("** NoAllocator::alloc() does not allocate memory **");
    }

    #[inline]
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        // I deny all allocations, so I don't need to free.
    }
}

#[cfg(feature = "alloc")]
mod alloc {
    use {
        crate::{entrypoint::MAX_HEAP_LENGTH, hint::unlikely},
        core::{
            alloc::{GlobalAlloc, Layout},
            mem::size_of,
            ptr::null_mut,
        },
    };

    /// The bump allocator used as the default Rust heap when running programs.
    ///
    /// The allocator uses a forward bump allocation strategy, where memory is
    /// allocated by moving a pointer forward in a pre-allocated memory
    /// region. The current position of the heap pointer is stored at the
    /// start of the memory region.
    ///
    /// This implementation relies on the runtime to zero out memory and to
    /// enforce the limit of the heap memory region. Use of memory outside
    /// the allocated region will result in a runtime error.
    #[cfg_attr(feature = "copy", derive(Copy))]
    #[derive(Clone, Debug)]
    pub struct BumpAllocator {
        start: usize,
        end: usize,
    }

    impl BumpAllocator {
        /// Creates the allocator tied to specific range of addresses.
        ///
        /// # Safety
        ///
        /// This is unsafe in most situations, unless you are totally sure that
        /// the provided start address and length can be written to by the
        /// allocator, and that the memory will be usable for the
        /// lifespan of the allocator. The start address must be aligned
        /// to `usize` and the length must be
        /// at least `size_of::<usize>()` bytes.
        ///
        /// For Solana on-chain programs, a certain address range is reserved,
        /// so the allocator can be given those addresses. In general,
        /// the `len` is set to the maximum heap length allowed by the
        /// runtime. The runtime will enforce the actual heap size
        /// requested by the program.
        pub const unsafe fn new_unchecked(start: usize, len: usize) -> Self {
            Self {
                start,
                end: start + len,
            }
        }
    }

    // Integer arithmetic in this global allocator implementation is safe when
    // operating on the prescribed `BumpAllocator::start` and
    // `BumpAllocator::end`. Any other use may overflow and is thus unsupported
    // and at one's own risk.
    #[allow(clippy::arithmetic_side_effects)]
    unsafe impl GlobalAlloc for BumpAllocator {
        /// Allocates memory as described by the given `layout` using a forward
        /// bump allocator.
        ///
        /// Returns a pointer to newly-allocated memory, or `null` to indicate
        /// allocation failure.
        ///
        /// # Safety
        ///
        /// `layout` must have non-zero size. Attempting to allocate for a
        /// zero-sized layout will result in undefined behavior.
        #[inline]
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            // Reads the current position of the heap pointer.
            //
            // Integer-to-pointer cast: the caller guarantees that `self.start` is a valid
            // address for the lifetime of the allocator and aligned to `usize`.
            let pos_ptr = self.start as *mut usize;
            let mut pos = *pos_ptr;

            if unlikely(pos == 0) {
                // First time, set starting position.
                pos = self.start + size_of::<usize>();
            }

            // Determines the allocation address, adjusting the alignment for the
            // type being allocated.
            let allocation = (pos + layout.align() - 1) & !(layout.align() - 1);

            if unlikely(layout.size() > MAX_HEAP_LENGTH as usize)
                || unlikely(self.end < allocation + layout.size())
            {
                return null_mut();
            }

            // Updates the heap pointer.
            *pos_ptr = allocation + layout.size();

            allocation as *mut u8
        }

        /// Behaves like `alloc`, but also ensures that the contents are set to
        /// zero before being returned.
        ///
        /// This method relies on the runtime to zero out the memory when
        /// reserving the heap region, so it simply calls `alloc`.
        #[inline]
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            self.alloc(layout)
        }

        /// This method has no effect since the bump allocator does not free
        /// memory.
        #[inline]
        unsafe fn dealloc(&self, _: *mut u8, _: Layout) {}
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::MAX_TX_ACCOUNTS,
        ::alloc::{
            alloc::{alloc, dealloc, handle_alloc_error},
            vec,
        },
        core::{
            alloc::Layout,
            hint::black_box,
            ptr::{copy_nonoverlapping, null_mut},
        },
    };

    /// The mock program ID used for testing.
    const MOCK_PROGRAM_ID: Address = Address::new_from_array([5u8; 32]);

    /// The mock instruction data used for testing.
    const MOCK_INSTRUCTION_DATA: [u8; 100] = [3u8; 100];

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
                    handle_alloc_error(layout);
                }
                AlignedMemory { ptr, layout }
            }
        }

        /// Write data to the memory region at the specified offset.
        ///
        /// # Safety
        ///
        /// The caller must ensure that the `data` length does not exceed the
        /// remaining space in the memory region starting from the
        /// `offset`.
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
    /// This function mimics the input buffer created by the SVM loader. Each
    /// account created has zeroed data, apart from the `data_len` field,
    /// which is set to the index of the account.
    ///
    /// # Safety
    ///
    /// The returned `AlignedMemory` should only be used within the test
    /// context.
    unsafe fn create_input(accounts: usize, instruction_data: &[u8]) -> (AlignedMemory, *mut u8) {
        let mut input = AlignedMemory::new(1_000_000_000);
        // Number of accounts.
        input.write(&(accounts as u64).to_le_bytes(), 0);
        let mut offset = size_of::<u64>();
        let mut pointers = vec![];

        for i in 0..accounts {
            pointers.push(offset);
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
        // Store the pointer to the instruction data.
        let instruction_data_ptr = input.as_mut_ptr().add(offset);
        // Instruction data.
        input.write(instruction_data, offset);
        offset += instruction_data.len();
        // Program ID (mock).
        input.write(MOCK_PROGRAM_ID.as_array(), offset);
        offset += size_of::<Address>();

        offset = (offset + (BPF_ALIGN_OF_U128 - 1)) & !(BPF_ALIGN_OF_U128 - 1);

        // Slice of account pointers.
        let accounts_ptr = input.as_mut_ptr().add(offset) as *mut AccountView;

        for (i, pointer) in pointers.into_iter().enumerate() {
            let account_ptr = input.as_mut_ptr().add(pointer) as *mut RuntimeAccount;
            accounts_ptr
                .add(i)
                .write(AccountView::new_unchecked(account_ptr));
        }

        (input, instruction_data_ptr)
    }

    /// Creates an input buffer with a specified number of accounts, including
    /// duplicated accounts, and instruction data.
    ///
    /// This function differs from `create_input` in that it creates accounts
    /// with a marker indicating that they are duplicated. There will be
    /// `accounts / 2` unique accounts, and the remaining `duplicated` accounts
    /// will be duplicates of the last unique account.
    ///
    /// This function mimics the input buffer created by the SVM loader. Each
    /// account created has zeroed data, apart from the `data_len` field,
    /// which is set to the index of the account.
    ///
    /// # Safety
    ///
    /// The returned `AlignedMemory` should only be used within the test
    /// context.
    unsafe fn create_input_with_duplicates(
        accounts: usize,
        instruction_data: &[u8],
        duplicated: usize,
    ) -> (AlignedMemory, *mut u8) {
        let mut input = AlignedMemory::new(1_000_000_000);
        // Number of accounts.
        input.write(&(accounts as u64).to_le_bytes(), 0);
        let mut offset = size_of::<u64>();
        let mut pointers = vec![];

        if accounts > 0 {
            assert!(
                duplicated < accounts,
                "Duplicated accounts must be less than total accounts"
            );
            let unique = accounts - duplicated;

            for i in 0..unique {
                pointers.push(offset);
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

            let last_unique_offset = *pointers.last().unwrap();

            // Remaining accounts are duplicated of the last unique account.
            for _ in unique..accounts {
                pointers.push(last_unique_offset);
                input.write(&[(unique - 1) as u8, 0, 0, 0, 0, 0, 0, 0], offset);
                offset += size_of::<u64>();
            }
        }

        // Instruction data length.
        input.write(&instruction_data.len().to_le_bytes(), offset);
        offset += size_of::<u64>();
        // Store the offset of the instruction data.
        let instruction_data_ptr = input.as_mut_ptr().add(offset);
        // Instruction data.
        input.write(instruction_data, offset);
        offset += instruction_data.len();
        // Program ID (mock).
        input.write(MOCK_PROGRAM_ID.as_array(), offset);
        offset += size_of::<Address>();

        offset = (offset + (BPF_ALIGN_OF_U128 - 1)) & !(BPF_ALIGN_OF_U128 - 1);

        // Slice of account pointers.
        let accounts_ptr = input.as_mut_ptr().add(offset) as *mut AccountView;

        for (i, pointer) in pointers.into_iter().enumerate() {
            let ptr = input.as_mut_ptr().add(pointer) as *mut RuntimeAccount;
            accounts_ptr.add(i).write(AccountView::new_unchecked(ptr));
        }

        (input, instruction_data_ptr)
    }

    /// Asserts that each account's data length matches its index.
    fn assert_accounts(
        program_id: &Address,
        accounts: &mut [AccountView],
        instruction_data: &[u8],
    ) -> ProgramResult {
        assert!(program_id == &MOCK_PROGRAM_ID);

        for (i, account) in accounts.iter().enumerate() {
            assert_eq!(account.data_len(), i);
        }

        assert_eq!(instruction_data, &MOCK_INSTRUCTION_DATA);

        Ok(())
    }

    /// Asserts that each account's data length matches its index for unique
    /// accounts, and that duplicated accounts reference the last unique
    /// account.
    fn assert_duplicated_accounts(
        program_id: &Address,
        accounts: &mut [AccountView],
        instruction_data: &[u8],
    ) -> ProgramResult {
        assert!(program_id == &MOCK_PROGRAM_ID);

        if !accounts.is_empty() {
            // Half of the accounts are duplicated.
            let unique = accounts.len() / 2;
            let (unique_accounts, duplicated_accounts) = accounts.split_at_mut(unique);

            // Unique accounts should have `data_len` equal to their index.
            for (i, account) in unique_accounts.iter().enumerate() {
                assert_eq!(account.data_len(), i);
            }

            // Last unique account.
            let last_unique = unique_accounts.last_mut().unwrap();

            // No mutable borrow active at this point.
            assert!(last_unique.try_borrow_mut().is_ok());

            // Duplicated accounts should reference (share) the account pointer
            // to the last unique account.
            for account in duplicated_accounts.iter_mut() {
                assert_eq!(account, last_unique);
                assert_eq!(account.data_len(), last_unique.data_len());

                let borrowed = account.try_borrow_mut().unwrap();
                // Only one mutable borrow at the same time should be allowed
                // on the duplicated account.
                assert!(last_unique.try_borrow_mut().is_err());
                black_box(borrowed);
            }

            // There should not be any mutable borrow on the duplicated account
            // at this point.
            assert!(last_unique.try_borrow_mut().is_ok());
        }

        assert_eq!(instruction_data, &MOCK_INSTRUCTION_DATA);

        Ok(())
    }

    #[test]
    fn test_process_entrypoint() {
        // Input with 0 accounts.

        let (mut program_input, instruction_data) =
            unsafe { create_input(0, &MOCK_INSTRUCTION_DATA) };

        unsafe {
            assert_eq!(
                process_entrypoint(
                    program_input.as_mut_ptr(),
                    instruction_data,
                    assert_accounts,
                ),
                0
            );
        };

        // Input with 3 accounts.

        let (mut program_input, instruction_data) =
            unsafe { create_input(3, &MOCK_INSTRUCTION_DATA) };

        unsafe {
            assert_eq!(
                process_entrypoint(
                    program_input.as_mut_ptr(),
                    instruction_data,
                    assert_accounts,
                ),
                0
            );
        };

        // Input with `MAX_TX_ACCOUNTS` accounts.

        let (mut program_input, instruction_data) =
            unsafe { create_input(MAX_TX_ACCOUNTS, &MOCK_INSTRUCTION_DATA) };

        unsafe {
            assert_eq!(
                process_entrypoint(
                    program_input.as_mut_ptr(),
                    instruction_data,
                    assert_accounts,
                ),
                0
            );
        }
    }

    #[test]
    fn test_deserialize_duplicated() {
        // Input with 0 accounts.

        let (mut program_input, instruction_data) =
            unsafe { create_input_with_duplicates(0, &MOCK_INSTRUCTION_DATA, 0) };

        unsafe {
            assert_eq!(
                process_entrypoint(
                    program_input.as_mut_ptr(),
                    instruction_data,
                    assert_duplicated_accounts,
                ),
                0
            );
        }

        // Input with 64 (32 duplicated) accounts.

        let (mut program_input, instruction_data) =
            unsafe { create_input_with_duplicates(64, &MOCK_INSTRUCTION_DATA, 32) };

        unsafe {
            assert_eq!(
                process_entrypoint(
                    program_input.as_mut_ptr(),
                    instruction_data,
                    assert_duplicated_accounts,
                ),
                0
            );
        }

        // Input with 250 (125 duplicated) accounts.

        let (mut program_input, instruction_data) =
            unsafe { create_input_with_duplicates(250, &MOCK_INSTRUCTION_DATA, 125) };

        unsafe {
            assert_eq!(
                process_entrypoint(
                    program_input.as_mut_ptr(),
                    instruction_data,
                    assert_duplicated_accounts,
                ),
                0
            );
        }
    }

    #[test]
    fn test_bump_allocator() {
        // alloc the entire
        {
            let mut heap = AlignedMemory::new(128);
            unsafe { heap.write(&[0; 128], 0) };

            let allocator = unsafe {
                BumpAllocator::new_unchecked(heap.as_mut_ptr() as usize, heap.layout.size())
            };

            for i in 0..128 - size_of::<*mut u8>() {
                let ptr = unsafe {
                    allocator.alloc(Layout::from_size_align(1, size_of::<u8>()).unwrap())
                };
                assert_eq!(
                    ptr as usize,
                    heap.as_mut_ptr() as usize + size_of::<*mut u8>() + i
                );
            }
            assert_eq!(null_mut(), unsafe {
                allocator.alloc(Layout::from_size_align(1, size_of::<u8>()).unwrap())
            });
        }
        // check alignment
        {
            let mut heap = AlignedMemory::new(128);
            unsafe { heap.write(&[0; 128], 0) };

            let allocator = unsafe {
                BumpAllocator::new_unchecked(heap.as_mut_ptr() as usize, heap.layout.size())
            };
            let ptr =
                unsafe { allocator.alloc(Layout::from_size_align(1, size_of::<u8>()).unwrap()) };
            assert_eq!(0, ptr.align_offset(size_of::<u8>()));
            let ptr =
                unsafe { allocator.alloc(Layout::from_size_align(1, size_of::<u16>()).unwrap()) };
            assert_eq!(0, ptr.align_offset(size_of::<u16>()));
            let ptr =
                unsafe { allocator.alloc(Layout::from_size_align(1, size_of::<u32>()).unwrap()) };
            assert_eq!(0, ptr.align_offset(size_of::<u32>()));
            let ptr =
                unsafe { allocator.alloc(Layout::from_size_align(1, size_of::<u64>()).unwrap()) };
            assert_eq!(0, ptr.align_offset(size_of::<u64>()));
            let ptr =
                unsafe { allocator.alloc(Layout::from_size_align(1, size_of::<u128>()).unwrap()) };
            assert_eq!(0, ptr.align_offset(size_of::<u128>()));
            let ptr = unsafe { allocator.alloc(Layout::from_size_align(1, 64).unwrap()) };
            assert_eq!(0, ptr.align_offset(64));
        }
        // alloc entire block (minus the pos ptr)
        {
            let mut heap = AlignedMemory::new(128);
            unsafe { heap.write(&[0; 128], 0) };

            let allocator = unsafe {
                BumpAllocator::new_unchecked(heap.as_mut_ptr() as usize, heap.layout.size())
            };
            let ptr = unsafe {
                allocator.alloc(
                    Layout::from_size_align(
                        heap.layout.size() - size_of::<usize>(),
                        size_of::<u8>(),
                    )
                    .unwrap(),
                )
            };
            assert_ne!(ptr, null_mut());
            assert_eq!(0, ptr.align_offset(size_of::<u64>()));
        }
    }
}
