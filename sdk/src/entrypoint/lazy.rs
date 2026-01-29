//! Defines the lazy program entrypoint and the context to access the
//! input buffer.

use crate::{
    account::{AccountView, RuntimeAccount},
    entrypoint::{NON_DUP_MARKER, STATIC_ACCOUNT_DATA},
    error::ProgramError,
    Address, BPF_ALIGN_OF_U128,
};

/// Declare the lazy program entrypoint.
///
/// This entrypoint is defined as *lazy* because it does not read the accounts
/// upfront. Instead, it provides an [`InstructionContext`] to the access input
/// information on demand. This is useful when the program needs more control
/// over the compute units it uses. The trade-off is that the program is
/// responsible for managing potential duplicated accounts and set up a `global
/// allocator` and `panic handler`.
///
/// The usual use-case for a [`crate::lazy_program_entrypoint!`] is small
/// programs with a single instruction. For most use-cases, it is recommended to
/// use the [`crate::program_entrypoint!`] macro instead.
///
/// This macro emits the boilerplate necessary to begin program execution,
/// calling a provided function to process the program instruction supplied by
/// the runtime, and reporting its result to the runtime. Note that it does not
/// set up a global allocator nor a panic handler.
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
/// Defining an entrypoint and making it conditional on the `bpf-entrypoint`
/// feature. Although the `entrypoint` module is written inline in this example,
/// it is common to put it into its own file.
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
///         Ok(())
///     }
///
/// }
/// ```
#[macro_export]
macro_rules! lazy_program_entrypoint {
    ( $process_instruction:expr ) => {
        /// Program entrypoint.
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            match $process_instruction($crate::entrypoint::lazy::InstructionContext::new_unchecked(
                input,
            )) {
                Ok(_) => $crate::SUCCESS,
                Err(error) => error.into(),
            }
        }
    };
}

/// Context to access data from the input buffer for the instruction.
///
/// This is a wrapper around the input buffer that provides methods to read the
/// accounts and instruction data. It is used by the lazy entrypoint to access
/// the input data on demand.
#[derive(Debug)]
pub struct InstructionContext {
    /// Pointer to the runtime input buffer to read from.
    ///
    /// This pointer is moved forward as accounts are read from the buffer.
    buffer: *mut u8,

    /// Number of remaining accounts.
    ///
    /// This value is decremented each time [`next_account`] is called.
    remaining: u64,
}

impl InstructionContext {
    /// Creates a new [`InstructionContext`] for the input buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the input buffer is valid, i.e., it
    /// represents the program input parameters serialized by the SVM
    /// loader. The SVM loader serializes the input parameters aligned to
    /// `8` bytes, with the first `8` bytes representing the number of
    /// accounts, followed by the accounts themselves, the instruction data
    /// and the program id.
    ///
    /// More information on the input buffer format can be found in the
    /// [SVM documentation].
    ///
    /// [SVM documentation]: https://solana.com/docs/programs/faq#input-parameter-serialization
    #[inline(always)]
    pub unsafe fn new_unchecked(input: *mut u8) -> Self {
        Self {
            // SAFETY: The first 8 bytes of the input buffer represent the
            // number of accounts when serialized by the SVM loader, which is read
            // when the context is created.
            buffer: unsafe { input.add(core::mem::size_of::<u64>()) },
            // SAFETY: Read the number of accounts from the input buffer serialized
            // by the SVM loader.
            remaining: unsafe { *(input as *const u64) },
        }
    }

    /// Reads the next account for the instruction.
    ///
    /// The account is represented as a [`MaybeAccount`], since it can either
    /// represent and [`AccountView`] or the index of a duplicated account. It
    /// is up to the caller to handle the mapping back to the source
    /// account.
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

        Ok(unsafe { self.read_account() })
    }

    /// Returns the next account for the instruction.
    ///
    /// Note that this method does *not* decrement the number of remaining
    /// accounts, but moves the input pointer forward. It is intended for
    /// use when the caller is certain on the number of remaining accounts.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee that there are remaining accounts;
    /// calling this when there are no more remaining accounts results in
    /// undefined behavior.
    #[inline(always)]
    pub unsafe fn next_account_unchecked(&mut self) -> MaybeAccount {
        self.read_account()
    }

    /// Returns the number of remaining accounts.
    ///
    /// This value is decremented each time [`Self::next_account`] is called.
    #[inline(always)]
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Returns the data for the instruction.
    ///
    /// This method can only be used after all accounts have been read;
    /// otherwise, it will return a [`ProgramError::InvalidInstructionData`]
    /// error.
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
    /// It is up to the caller to guarantee that all accounts have been read;
    /// calling this method before reading all accounts will result in
    /// undefined behavior.
    #[inline(always)]
    pub unsafe fn instruction_data_unchecked(&self) -> &[u8] {
        let data_len = *(self.buffer as *const usize);
        // shadowing the input to avoid leaving it in an inconsistent position
        let data = self.buffer.add(core::mem::size_of::<u64>());
        core::slice::from_raw_parts(data, data_len)
    }

    /// Returns the program id for the instruction.
    ///
    /// This method can only be used after all accounts have been read;
    /// otherwise, it will return a [`ProgramError::InvalidInstructionData`]
    /// error.
    #[inline(always)]
    pub fn program_id(&self) -> Result<&Address, ProgramError> {
        if self.remaining > 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(unsafe { self.program_id_unchecked() })
    }

    /// Returns the program id for the instruction.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee that all accounts have been read;
    /// calling this method before reading all accounts will result in
    /// undefined behavior.
    #[inline(always)]
    pub unsafe fn program_id_unchecked(&self) -> &Address {
        let data_len = *(self.buffer as *const usize);
        &*(self.buffer.add(core::mem::size_of::<u64>() + data_len) as *const Address)
    }

    /// Read an account from the input buffer.
    ///
    /// This can only be called with a buffer that was serialized by the runtime
    /// as it assumes a specific memory layout.
    #[allow(clippy::cast_ptr_alignment, clippy::missing_safety_doc)]
    #[inline(always)]
    unsafe fn read_account(&mut self) -> MaybeAccount {
        let account: *mut RuntimeAccount = self.buffer as *mut RuntimeAccount;
        // Adds an 8-bytes offset for:
        //   - rent epoch in case of a non-duplicate account
        //   - duplicate marker + 7 bytes of padding in case of a duplicate account
        self.buffer = self.buffer.add(core::mem::size_of::<u64>());

        if (*account).borrow_state == NON_DUP_MARKER {
            self.buffer = self.buffer.add(STATIC_ACCOUNT_DATA);
            self.buffer = self.buffer.add((*account).data_len as usize);
            self.buffer = self.buffer.add(self.buffer.align_offset(BPF_ALIGN_OF_U128));

            MaybeAccount::Account(AccountView::new_unchecked(account))
        } else {
            // The caller will handle the mapping to the original account.
            MaybeAccount::Duplicated((*account).borrow_state)
        }
    }
}

/// Wrapper type around an [`AccountView`] that may be a duplicate.
#[cfg_attr(feature = "copy", derive(Copy))]
#[derive(Debug, Clone)]
pub enum MaybeAccount {
    /// An [`AccountView`] that is not a duplicate.
    Account(AccountView),

    /// The index of the original account that was duplicated.
    Duplicated(u8),
}

impl MaybeAccount {
    /// Extracts the wrapped [`AccountView`].
    ///
    /// It is up to the caller to guarantee that the [`MaybeAccount`] really is
    /// in an [`MaybeAccount::Account`]. Calling this method when the
    /// variant is a [`MaybeAccount::Duplicated`] will result in a panic.
    #[inline(always)]
    pub fn assume_account(self) -> AccountView {
        let MaybeAccount::Account(account) = self else {
            panic!("Duplicated account")
        };
        account
    }
}
