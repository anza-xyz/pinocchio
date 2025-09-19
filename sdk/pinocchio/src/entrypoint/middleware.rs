//! Defines the middleware program entrypoint, enabling a hot path to bypass
//! entrypoint deserialization, ejecting to the cold path on failure.

/// Declare the middleware program entrypoint.
///
/// The macro expects a `hot` and `cold` path expressions. The `hot` is a function with
/// the following type signature:
///
/// ```ignore
/// fn hot(input: mut *u8) -> u64;
/// ```
/// The `input` argument represents the program input parameters serialized by the SVM
/// loader. The `cold` is a function with the following type signature:
///
/// ```ignore
/// fn cold(
///     program_id: &Pubkey,      // Public key of the account the program was loaded into
///     accounts: &[AccountInfo], // All accounts required to process the instruction
///     instruction_data: &[u8],  // Serialized instruction-specific data
/// ) -> ProgramResult;
/// ```
/// # Example
///
/// A middleware program entrypoint where an invocation with zero accounts will lead to
/// the "hot" path, and anything else will fallback to the "cold" path:
/// 
///  ```norun
/// #![cfg_attr(target_os = "solana", no_std)]
/// use pinocchio::{
///     ProgramResult,
///     account_info::AccountInfo,
///     middleware_program_entrypoint,
///     msg,
///     no_allocator,
///     nostd_panic_handler,
///     pubkey::Pubkey
/// };
/// 
/// nostd_panic_handler!();
/// no_allocator!();
/// 
/// middleware_program_entrypoint!(hot,cold);
/// 
/// // This uses 4 CUs
/// #[inline(always)]
/// pub fn hot(input: *mut u8) -> u64 {
///     unsafe { *input as u64 }
/// }
/// 
/// // This uses 113 CUs
/// #[cold]
/// #[inline(always)]
/// pub fn cold(
///     _program_id: &Pubkey,
///     _accounts: &[AccountInfo],
///     _instruction_data: &[u8],
/// ) -> ProgramResult {
///     msg!("Hello from Pinocchio!");
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! middleware_program_entrypoint {
    ($hot:expr, $cold:expr) => {
        $crate::middleware_program_entrypoint!($hot, $cold, { $crate::MAX_TX_ACCOUNTS });
    };
    ($hot:expr, $cold:expr, $maximum:expr ) => {

        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            if $hot(input) == 0 {
                return $crate::SUCCESS
            }

            const UNINIT: core::mem::MaybeUninit<$crate::account_info::AccountInfo> = core::mem::MaybeUninit::<$crate::account_info::AccountInfo>::uninit();
            // Create an array of uninitialized account infos.
            let mut accounts = [UNINIT; $maximum];

            let (program_id, count, instruction_data) = unsafe {
                $crate::entrypoint::deserialize::<$maximum>(input, &mut accounts) }; 

            // Call the program's entrypoint passing `count` account infos; we know that
            // they are initialized so we cast the pointer to a slice of `[AccountInfo]`.
            match $cold(
                &program_id,
                unsafe { core::slice::from_raw_parts(accounts.as_ptr() as _, count) },
                &instruction_data,
            ) {
                Ok(()) => $crate::SUCCESS,
                Err(error) => error.into(),
            }
        }
    };
}

#[cfg(feature = "asm")]
#[macro_export]
macro_rules! asm_middleware_program_entrypoint {
    ($hot:expr, $cold:expr) => {
        $crate::asm_middleware_program_entrypoint!($hot, $cold, { $crate::MAX_TX_ACCOUNTS });
    };
    ($hot:expr, $cold:expr, $maximum:expr ) => {

        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) {
            // Inject `hot()` directly. Assume conditional return is handled within this function.
            $hot(input);
            const UNINIT: core::mem::MaybeUninit<$crate::account_info::AccountInfo> = core::mem::MaybeUninit::<$crate::account_info::AccountInfo>::uninit();
            // Create an array of uninitialized account infos.
            let mut accounts = [UNINIT; $maximum];

            let (program_id, count, instruction_data) = unsafe {
                $crate::entrypoint::deserialize::<$maximum>(input, &mut accounts) }; 

            // Call the program's entrypoint passing `count` account infos; we know that
            // they are initialized so we cast the pointer to a slice of `[AccountInfo]`.
            match $cold(
                &program_id,
                unsafe { core::slice::from_raw_parts(accounts.as_ptr() as _, count) },
                &instruction_data,
            ) {
                Ok(()) => {
                    $crate::set_return_imm!($crate::SUCCESS)
                },
                Err(error) => {
                    $crate::set_return!(Into::<u64>::into(error) as u32)
                },
            }
        }
    };
}

/// # Set Return
/// Sets the return register `r0` to a stored value. For static return values, consider using `set_return_imm` to save 1 CU by avoiding an additional register allocation.
///
/// ### CU Cost
/// 1 CU (+1 CU for register allocation)
/// 
/// ### ASM
/// `mov32 r0, r1`
/// 
/// ### Parameters
/// - `value`: The stored value to set the return register.
///
/// ### Example
/// ```
/// let n = 1337;
/// set_return(n);
/// ```
///
/// This will assign `1337` to `n`, then set the register `r0` to `n`.
#[cfg(feature = "asm")]
#[macro_export]
macro_rules! set_return {
    ($value:expr) => {
        #[cfg(target_os = "solana")]
        unsafe {
            core::arch::asm!(
                "mov32 r0, {0}",
                in(reg) $value
            );
        }
    };
}

/// # Set Return Register from Immediate
/// Sets the return register `r0` to an immediate value.
///
/// ### CU Cost
/// 1 CU
/// 
/// ### ASM
/// `mov32 r0, r1`
/// 
/// ### Parameters
/// - `value`: The stored value to set the return register.
///
/// ### Example
/// ```
/// set_return_imm(1337); // Set return value to error code 1
/// ```
///
/// This will assign the return register `r0` to `1337`.
#[cfg(feature = "asm")]
#[macro_export]
macro_rules! set_return_imm {
    ($value:expr) => {
        #[cfg(target_os = "solana")]
        unsafe {
            core::arch::asm!(
                "mov32 r0, {0}",
                const $value
            );
        }
    };
}

/// # Exit
/// Immediately exits the program
///
/// ### CU Cost
/// 1 CU
/// 
/// ### ASM
/// `exit`
/// 
/// ### Example
/// ```
/// exit!(); // Set return value to error code 1
/// ```
#[cfg(feature = "asm")]
#[macro_export]
macro_rules! exit {
    () => {
        #[cfg(target_os = "solana")]
        unsafe {
            core::arch::asm!(
                "exit"
            );
        }
    }
}

/// # Reset
/// Resets r1 to the address of the serialized input region
///
/// ### CU Cost
/// 1 CU
/// 
/// ### ASM
/// `lddw r1, 0x400000000`
/// 
/// ### Example
/// ```
/// reset!(); // Set r1 to address of serialized input region
/// ```
#[cfg(feature = "asm")]
#[macro_export]
macro_rules! reset {
    () => {
        #[cfg(target_os = "solana")]
        unsafe {
            core::arch::asm!(
                "lddw r1, 0x400000000"
            );
        }
    }
}