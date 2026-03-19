//! Defines the inline program entrypoint and associated types.

use {core::slice::from_raw_parts, solana_address::Address};

/// Maximum number of accounts that can be parsed inline in the entrypoint.
///
/// This is a hard limit to constrain the size of the parsing logic
/// inlined in the entrypoint.
pub const MAX_INLINE_ACCOUNTS: usize = 10;

/// Declare the inline program entrypoint.
///
/// This entrypoint is defined as *inline* because it inlines the account
/// parsing logic into the entrypoint itself. It takes advantage of the fact
/// that the runtime passes the instruction data pointer to the entrypoint.
/// This allows the entrypoint to read the instruction data and decide how many
/// accounts it wants to parse from the program input.
///
/// It offers two macros to declare the entrypoint:
/// [`crate::inline_program_entrypoint!`] and [`crate::execute!`]. The former is
/// used to declare the entrypoint and the latter is used to execute the program
/// logic with a specified number of accounts and a processor function. The
/// processor function is called with the program id, the accounts, and the
/// instruction data.
///
/// The [`crate::inline_program_entrypoint!`] macro is used to declare the
/// entrypoint. The only argument is the name of a function with this type
/// signature:
///
/// ```ignore
/// fn process_instruction(input: ProgramInput) -> ProgramResult;
/// ```
///
/// [`ProgramInput`] offers a method to read the instruction data. Programs can
/// then use the [`crate::execute!`] macro to execute the program logic for the
/// corresponding instruction. The [`crate::execute!`] macro takes three
/// arguments: the number of accounts to parse from the input buffer, the
/// program input, and the name of the processor function. The processor
/// function has this type signature:
///
/// ```ignore
/// fn processor(
///     program_id: &Address,
///     accounts: &mut [AccountView],
///     instruction_data: &[u8]
///  ) -> ProgramResult;
/// ```
///
/// # Example
///
/// Defining an entrypoint and making it conditional on the `bpf-entrypoint`
/// feature. Although the `entrypoint` module is written inline in this example,
/// it is common to put it into its own file.
///
/// ```no_run
/// #[cfg(feature = "bpf-entrypoint")]
/// pub mod entrypoint {
///     use {
///         pinocchio::{
///             entrypoint::inline::ProgramInput, error::ProgramError, execute,
///             inline_program_entrypoint, AccountView, Address, ProgramResult,
///         },
///         pinocchio_system::instructions::CreateAccount,
///     };
///
///     // Declares the entrypoint of the program.
///     inline_program_entrypoint!(process_instruction);
///
///     pub fn process_instruction(input: ProgramInput) -> ProgramResult {
///         match input.data.first() {
///            Some(&0) => execute!((3, input) => create),
///            _ => return Err(ProgramError::InvalidInstructionData),
///        }
///     }
///
///     /// Instruction processor.
///     pub fn create(
///         program_id: &Address,
///         accounts: &mut [AccountView],
///         _instruction_data: &[u8],
///     ) -> ProgramResult {
///         let [from, to, _system_program] = accounts else {
///             return Err(ProgramError::NotEnoughAccountKeys);
///         };
///
///         CreateAccount {
///             from,
///             to,
///             lamports: 1_000_000_000,
///             space: 10,
///             owner: program_id,
///         }
///         .invoke()
///     }
/// }
/// ```
#[macro_export]
macro_rules! inline_program_entrypoint {
    ( $process_instruction:expr ) => {
        /// Program entrypoint.
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(
            program_input: *mut u8,
            instruction_data: *const u8,
        ) -> u64 {
            match $process_instruction($crate::entrypoint::inline::ProgramInput::new_unchecked(
                program_input,
                instruction_data,
            )) {
                Ok(_) => $crate::SUCCESS,
                Err(error) => error.into(),
            }
        }
    };
}

/// Representation of the program input parameters passed by the runtime to the
/// entrypoint.
#[derive(Debug)]
pub struct ProgramInput {
    /// The data for the instruction.
    pub data: &'static [u8],

    /// Pointer to the runtime input buffer to read from.
    pub raw: *mut u8,

    /// Number of available accounts.
    pub available: u64,
}

impl ProgramInput {
    /// Creates a new [`ProgramInput`] for the input buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that both `program_input` and `instruction_data`
    /// are valid pointers to the correct locations in the input buffer as
    /// serialized by the SVM loader. The `program_input` pointer should point
    /// to the start of the input buffer, and the `instruction_data` pointer
    /// should point to the start of the instruction data within that buffer.
    #[inline(always)]
    pub unsafe fn new_unchecked(program_input: *mut u8, instruction_data: *const u8) -> Self {
        // SAFETY: The 8 bytes preceding the instruction data represent the instruction
        // data length.
        let length =
            unsafe { *(instruction_data.sub(::core::mem::size_of::<u64>()) as *const u64) };

        Self {
            data: unsafe { from_raw_parts(instruction_data, length as usize) },
            // SAFETY: Account data is located after the first 8 bytes on the program input.
            raw: unsafe { program_input.add(::core::mem::size_of::<u64>()) },
            // SAFETY: The number of accounts is serialized at the start of the program
            // input buffer by the SVM loader.
            available: unsafe { *(program_input as *const u64) },
        }
    }

    /// Return the address of the program.
    pub fn program_id(&self) -> &'static Address {
        // SAFETY: The program id is located at the end of the program input buffer
        // serialized by the SVM loader after the instruction data.
        unsafe { &*self.data.as_ptr().add(self.data.len()).cast::<Address>() }
    }
}

#[macro_export]
macro_rules! execute {
    ( (1, $context:expr) => $processor:expr ) => {{
        if $context.available < 1 {
            return $crate::error::ProgramError::NotEnoughAccountKeys;
        }

        let mut accounts = [const { ::core::mem::MaybeUninit::<$crate::AccountView>::uninit() }; $crate::MAX_TX_ACCOUNTS];

        let mut ptr = accounts.as_mut_ptr() as *mut $crate::AccountView;
        let accounts_slice = ptr;

        let mut input = $context.raw;

        let account = input as *mut $crate::account::RuntimeAccount;

        // SAFETY: First account is always non-duplicated.
        unsafe {
            $crate::__store_data_length_in_padding!(account);
            ptr.write($crate::AccountView::new_unchecked(account));

            input = input.add(size_of::<u64>());
            $crate::__advance_input_with_account!(input, account);
        }

        let accounts = unsafe {
            ::core::slice::from_raw_parts_mut(accounts.as_mut_ptr() as *mut $crate::AccountView, 1)
        };

        $processor($context.program_id(), accounts, $context.data)
    }};

    ( (2, $context:expr) => $processor:expr ) => {
        $crate::execute!(@impl 2, 1, $context, $processor)
    };

    ( (3, $context:expr) => $processor:expr) => {
        $crate::execute!(@impl 3, 2, $context, $processor)
    };

    ( (4, $context:expr) => $processor:expr ) => {
        $crate::execute!(@impl 4, 3, $context, $processor)
    };

    ( (5, $context:expr) => $processor:expr) => {
        $crate::execute!(@impl 5, 4, $context, $processor)
    };

    ( (6, $context:expr) => $processor:expr) => {
        $crate::execute!(@impl 6, 5, $context, $processor)
    };

    ( (7, $context:expr) => $processor:expr) => {
        $crate::execute!(@impl 7, 6, $context, $processor)
    };

    ( (8, $context:expr) => $processor:expr) => {
        $crate::execute!(@impl 8, 7, $context, $processor)
    };

    ( (9, $context:expr) => $processor:expr) => {
        $crate::execute!(@impl 9, 8, $context, $processor)
    };

    ( (10, $context:expr) => $processor:expr) => {
        $crate::execute!(@impl 10, 9, $context, $processor)
    };

    // Shared implementation
    (@impl $n:tt, $remaining:tt, $context:expr, $processor:expr) => {{
        if $crate::hint::unlikely($context.available < $n) {
            return Err($crate::error::ProgramError::NotEnoughAccountKeys);
        }

        let mut accounts = [const { ::core::mem::MaybeUninit::<$crate::AccountView>::uninit() }; $crate::MAX_TX_ACCOUNTS];

        let mut ptr = accounts.as_mut_ptr() as *mut $crate::AccountView;
        let accounts_slice = ptr;

        let mut input = $context.raw;

        let account = input as *mut $crate::account::RuntimeAccount;

        // SAFETY: First account is always non-duplicated.
        unsafe {
            $crate::__store_data_length_in_padding!(account);
            ptr.write($crate::AccountView::new_unchecked(account));

            input = input.add(size_of::<u64>());
            $crate::__advance_input_with_account!(input, account);

            $crate::__process_accounts!($remaining => (input, ptr, accounts_slice));
        }

        let accounts = unsafe {
            ::core::slice::from_raw_parts_mut(accounts.as_mut_ptr() as *mut $crate::AccountView, $n)
        };

        $processor($context.program_id(), accounts, $context.data)
    }};
}
