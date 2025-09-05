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