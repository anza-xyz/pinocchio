//! Defines the lazy program entrypoint and the context to access the
//! input buffer.

use crate::{
    account_info::{Account, AccountInfo, MAX_PERMITTED_DATA_INCREASE},
    program_error::ProgramError,
    pubkey::Pubkey,
    BPF_ALIGN_OF_U128, NON_DUP_MARKER,
};

/// Declare the lazy program entrypoint.
///
/// Use the `lazy_program_entrypoint!` macro instead.
#[deprecated(
    since = "0.7.0",
    note = "Use the `lazy_program_entrypoint!` macro instead"
)]
#[macro_export]
macro_rules! lazy_entrypoint {
    ( $process_instruction:ident ) => {
        $crate::lazy_program_entrypoint!($process_instruction);
    };
}

/// Declare the lazy program entrypoint.
///
/// This entrypoint is defined as *lazy* because it does not read the accounts upfront.
/// Instead, it provides an [`InstructionContext`] to the access input information on demand.
/// This is useful when the program needs more control over the compute units it uses.
/// The trade-off is that the program is responsible for managing potential duplicated
/// accounts and set up a `global allocator` and `panic handler`.
///
/// The usual use-case for a [`crate::lazy_program_entrypoint!`] is small programs with a single
/// instruction. For most use-cases, it is recommended to use the [`crate::program_entrypoint!`]
/// macro instead.
///
/// This macro emits the boilerplate necessary to begin program execution, calling a
/// provided function to process the program instruction supplied by the runtime, and reporting
/// its result to the runtime. Note that it does not set up a global allocator nor a panic
/// handler.
///
/// The only argument is the name of a function with this type signature:
///
/// ```ignore
/// fn process_instruction(
///    mut context: InstructionContext, // wrapper around the input buffer
/// ) -> ProgramResult;
/// ```
///
/// # Example
///
/// Defining an entrypoint and making it conditional on the `bpf-entrypoint` feature. Although
/// the `entrypoint` module is written inline in this example, it is common to put it into its
/// own file.
///
/// ```no_run
/// #[cfg(feature = "bpf-entrypoint")]
/// pub mod entrypoint {
///
///     use pinocchio::{
///         default_allocator,
///         default_panic_handler,
///         entrypoint::InstructionContext,
///         lazy_program_entrypoint,
///         msg,
///         ProgramResult
///     };
///
///     lazy_program_entrypoint!(process_instruction);
///     default_allocator!();
///     default_panic_handler!();
///
///     pub fn process_instruction(
///         mut context: InstructionContext,
///     ) -> ProgramResult {
///         msg!("Hello from my `lazy` program!");
///         Ok(())
///     }
///
/// }
/// ```
#[macro_export]
macro_rules! lazy_program_entrypoint {
    ( $process_instruction:ident ) => {
        /// Program entrypoint.
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            match $process_instruction($crate::entrypoint::lazy::InstructionContext::new(input)) {
                Ok(_) => $crate::SUCCESS,
                Err(error) => error.into(),
            }
        }
    };
}

/// Context to access data from the input buffer for the instruction.
///
/// This is a wrapper around the input buffer that provides methods to read the accounts
/// and instruction data. It is used by the lazy entrypoint to access the input data on demand.
pub struct InstructionContext {
    /// Pointer to the runtime input buffer for the instruction.
    input: *mut u8,

    /// Number of remaining accounts.
    ///
    /// This value is decremented each time [`next_account`] is called.
    remaining: u64,

    /// Current memory offset on the input buffer.
    offset: usize,
}

impl InstructionContext {
    /// Creates a new [`InstructionContext`] for the input buffer.
    #[inline(always)]
    pub fn new(input: *mut u8) -> Self {
        Self {
            input,
            remaining: unsafe { *(input as *const u64) },
            offset: core::mem::size_of::<u64>(),
        }
    }

    /// Reads the next account for the instruction.
    ///
    /// The account is represented as a [`MaybeAccount`], since it can either
    /// represent and [`AccountInfo`] or the index of a duplicated account. It is up to the
    /// caller to handle the mapping back to the source account.
    ///
    /// # Error
    ///
    /// Returns a [`ProgramError::NotEnoughAccountKeys`] error if there are
    /// no remaining accounts.
    #[inline(always)]
    pub fn next_account(&mut self) -> Result<MaybeAccount, ProgramError> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(ProgramError::NotEnoughAccountKeys)?;

        Ok(unsafe { read_account(self.input, &mut self.offset) })
    }

    /// Returns the next account for the instruction.
    ///
    /// Note that this method does *not* decrement the number of remaining accounts, but moves
    /// the offset forward. It is intended for use when the caller is certain on the number of
    /// remaining accounts.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee that there are remaining accounts; calling this when
    /// there are no more remaining accounts results in undefined behavior.
    #[inline(always)]
    pub unsafe fn next_account_unchecked(&mut self) -> MaybeAccount {
        read_account(self.input, &mut self.offset)
    }

    /// Returns the number of available accounts.
    #[inline(always)]
    pub fn available(&self) -> u64 {
        unsafe { *(self.input as *const u64) }
    }

    /// Returns the number of remaining accounts.
    ///
    /// This value is decremented each time [`Self::next_account`] is called.
    #[inline(always)]
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Returns the instruction data for the instruction.
    ///
    /// This method can only be used after all accounts have been read; otherwise, it will
    /// return a [`ProgramError::InvalidInstructionData`] error.
    #[inline(always)]
    pub fn instruction_data(&self) -> Result<&[u8], ProgramError> {
        if self.remaining > 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(unsafe { self.instruction_data_unchecked() })
    }

    /// Returns the instruction data for the instruction.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee that all accounts have been read; calling this method
    /// before reading all accounts will result in undefined behavior.
    #[inline(always)]
    pub unsafe fn instruction_data_unchecked(&self) -> &[u8] {
        let data_len = *(self.input.add(self.offset) as *const usize);
        // shadowing the offset to avoid leaving it in an inconsistent state
        let offset = self.offset + core::mem::size_of::<u64>();
        core::slice::from_raw_parts(self.input.add(offset), data_len)
    }

    /// Returns the program id for the instruction.
    ///
    /// This method can only be used after all accounts have been read; otherwise, it will
    /// return a [`ProgramError::InvalidInstructionData`] error.
    #[inline(always)]
    pub fn program_id(&self) -> Result<&Pubkey, ProgramError> {
        if self.remaining > 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(unsafe { self.program_id_unchecked() })
    }

    /// Returns the program id for the instruction.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee that all accounts have been read; calling this method
    /// before reading all accounts will result in undefined behavior.
    #[inline(always)]
    pub unsafe fn program_id_unchecked(&self) -> &Pubkey {
        let data_len = *(self.input.add(self.offset) as *const usize);
        &*(self
            .input
            .add(self.offset + core::mem::size_of::<u64>() + data_len) as *const Pubkey)
    }
}

/// Wrapper type around an [`AccountInfo`] that may be a duplicate.
pub enum MaybeAccount {
    /// An [`AccountInfo`] that is not a duplicate.
    Account(AccountInfo),

    /// The index of the original account that was duplicated.
    Duplicated(u8),
}

impl MaybeAccount {
    /// Extracts the wrapped [`AccountInfo`].
    ///
    /// It is up to the caller to guarantee that the [`MaybeAccount`] really is in an
    /// [`MaybeAccount::Account`]. Calling this method when the variant is a
    /// [`MaybeAccount::Duplicated`] will result in a panic.
    #[inline(always)]
    pub fn assume_account(self) -> AccountInfo {
        let MaybeAccount::Account(account) = self else {
            panic!("Duplicated account")
        };
        account
    }
}

/// Read an account from the input buffer.
///
/// This can only be called with a buffer that was serialized by the runtime as
/// it assumes a specific memory layout.
#[allow(clippy::cast_ptr_alignment, clippy::missing_safety_doc)]
#[inline(always)]
unsafe fn read_account(input: *mut u8, offset: &mut usize) -> MaybeAccount {
    let account: *mut Account = input.add(*offset) as *mut _;

    if (*account).borrow_state == NON_DUP_MARKER {
        // repurpose the borrow state to track borrows
        (*account).borrow_state = 0b_0000_0000;

        *offset += core::mem::size_of::<Account>();
        *offset += (*account).data_len as usize;
        *offset += MAX_PERMITTED_DATA_INCREASE;
        *offset += (*offset as *const u8).align_offset(BPF_ALIGN_OF_U128);
        *offset += core::mem::size_of::<u64>();

        MaybeAccount::Account(AccountInfo { raw: account })
    } else {
        *offset += core::mem::size_of::<u64>();
        //the caller will handle the mapping to the original account
        MaybeAccount::Duplicated((*account).borrow_state)
    }
}
