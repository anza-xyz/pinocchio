//! Macro for defining a custom program entrypoint

/// Declare a custom program entrypoint.
///
/// This macro is similar to the [`crate::program_entrypoint!`] macro, but it defines a custom entrypoint
/// function for use in a cold path fallback function. This is useful when the program requires fast path
/// optimization.
///
/// The first argument is the name of the custom entrypoint function.
///
/// The second argument is a function with this type signature:
///
/// ```ignore
/// fn process_instruction(
///     program_id: &Address,     // Address of the account the program was loaded into
///     accounts: &[AccountView], // All accounts required to process the instruction
///     instruction_data: &[u8],  // Serialized instruction-specific data
/// ) -> ProgramResult;
/// ```
/// The argument is defined as an `expr`, which allows the use of any function pointer not just
/// identifiers in the current scope.
///
/// There is a third optional argument that allows to specify the maximum number of accounts
/// expected by instructions of the program. This is useful to reduce the stack size requirement for
/// the entrypoint, as the default is set to [`MAX_TX_ACCOUNTS`]. If the program receives more
/// accounts than the specified maximum, these accounts will be ignored.
#[macro_export]
macro_rules! custom_program_entrypoint {
    ( $custom_entrypoint:ident, $process_instruction:expr ) => {
        $crate::custom_program_entrypoint!($custom_entrypoint, $process_instruction, { $crate::MAX_TX_ACCOUNTS });
    };
    ( $custom_entrypoint:ident, $process_instruction:expr, $maximum:expr ) => {
        /// Custom Program entrypoint.
        #[inline(always)]
        fn $custom_entrypoint(input: *mut u8) -> u64 {
            unsafe {
                $crate::entrypoint::entrypoint_deserialize::<$maximum>(input, $process_instruction)
            }
        }
    };
}
