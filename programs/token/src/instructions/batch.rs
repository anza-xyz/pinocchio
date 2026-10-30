#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use {
    crate::{instructions::invalid_argument_error, TokenInterface},
    core::{marker::PhantomData, mem::MaybeUninit, slice::from_raw_parts},
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_unchecked, CpiAccount, Signer, MAX_CPI_ACCOUNTS},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// The instruction discriminator.
const DISCRIMINATOR: u8 = 255;

/// The maximum instruction data buffer length required for a batch.
const MAX_DATA_LEN: usize = 10 * 1024;

/// The maximum account buffer length required for a batch.
const MAX_ACCOUNTS_LEN: usize = MAX_CPI_ACCOUNTS;

/// The size of the batch instruction header.
///
/// The header of each instruction consists of two `u8` values:
///   - number of the accounts
///   - length of the instruction data
const IX_HEADER_SIZE: usize = 2;

/// A collection of instructions that can be serialized into a token `Batch`
/// instruction.
pub struct Batch<'account, 'state, Program: TokenInterface> {
    /// The instruction data for the batch instruction.
    ///
    /// The first byte is reserved for the batch instruction discriminator,
    /// and each instruction's data is prefixed with a byte indicating the
    /// number of instruction accounts and a byte indicating the length of
    /// the instruction data.
    data: &'state mut [MaybeUninit<u8>],

    /// The instruction accounts for the batch instruction.
    instruction_accounts: &'state mut [MaybeUninit<InstructionAccount<'account>>],

    /// The accounts for the batch instruction.
    accounts: &'state mut [MaybeUninit<CpiAccount<'account>>],

    /// The current length of the instruction data.
    data_len: usize,

    /// The current length of the accounts.
    accounts_len: usize,

    /// The current length of the instruction accounts.    
    instruction_accounts_len: usize,

    /// Phantom data for the program.
    _program: PhantomData<Program>,
}

impl<'account, 'state, Program: TokenInterface> Batch<'account, 'state, Program>
where
    'account: 'state,
{
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = DISCRIMINATOR;

    /// The maximum instruction data buffer length required for a batch.
    pub const MAX_DATA_LEN: usize = MAX_DATA_LEN;

    /// The maximum account buffer length required for a batch.
    pub const MAX_ACCOUNTS_LEN: usize = MAX_ACCOUNTS_LEN;

    /// The size of the batch instruction header.
    pub const IX_HEADER_SIZE: usize = IX_HEADER_SIZE;

    /// Creates a new `Batch` with the provided buffers.
    #[inline(always)]
    pub fn new(
        data: &'state mut [MaybeUninit<u8>],
        instruction_accounts: &'state mut [MaybeUninit<InstructionAccount<'account>>],
        accounts: &'state mut [MaybeUninit<CpiAccount<'account>>],
    ) -> Result<Self, ProgramError> {
        if data.is_empty() {
            return Err(invalid_argument_error());
        }

        // The first byte of the instruction data is reserved for
        // the batch discriminator.
        data[0].write(DISCRIMINATOR);

        Ok(Self {
            data,
            instruction_accounts,
            accounts,
            data_len: 1,
            accounts_len: 0,
            instruction_accounts_len: 0,
            _program: PhantomData,
        })
    }

    /// Invokes the instruction with `Program::ID` without checking the
    /// borrow state of the accounts.
    ///
    /// # Safety
    ///
    /// This method does not perform any account borrow checking. The borrow
    /// state of the accounts is only checked when an instruction is added to
    /// the batch (e.g., via [`IntoBatch::into_batch`]), not when the batch is
    /// invoked.
    ///
    /// The caller must ensure that none of the batch accounts are borrowed
    /// between adding an instruction to the batch and invoking the batch.
    /// Otherwise, the CPI may modify account data or lamports while a
    /// reference to them is still alive, which is undefined behavior.
    #[inline(always)]
    pub unsafe fn invoke_unchecked(&self) -> ProgramResult {
        self.invoke_with_unverified_program_unchecked(&Program::ID)
    }

    /// Invokes the instruction with `Program::ID` and signer seeds without
    /// checking the borrow state of the accounts.
    ///
    /// # Safety
    ///
    /// This method does not perform any account borrow checking. The borrow
    /// state of the accounts is only checked when an instruction is added to
    /// the batch (e.g., via [`IntoBatch::into_batch`]), not when the batch is
    /// invoked.
    ///
    /// The caller must ensure that none of the batch accounts are borrowed
    /// between adding an instruction to the batch and invoking the batch.
    /// Otherwise, the CPI may modify account data or lamports while a
    /// reference to them is still alive, which is undefined behavior.
    #[inline(always)]
    pub unsafe fn invoke_signed_unchecked(&self, signers: &[Signer]) -> ProgramResult {
        self.invoke_signed_with_unverified_program_unchecked(signers, &Program::ID)
    }

    /// Invokes the instruction after verifying the `program` address without
    /// checking the borrow state of the accounts.
    ///
    /// # Safety
    ///
    /// This method does not perform any account borrow checking. The borrow
    /// state of the accounts is only checked when an instruction is added to
    /// the batch (e.g., via [`IntoBatch::into_batch`]), not when the batch is
    /// invoked.
    ///
    /// The caller must ensure that none of the batch accounts are borrowed
    /// between adding an instruction to the batch and invoking the batch.
    /// Otherwise, the CPI may modify account data or lamports while a
    /// reference to them is still alive, which is undefined behavior.
    #[inline(always)]
    pub unsafe fn invoke_with_program_unchecked(&self, program: &Address) -> ProgramResult {
        self.invoke_signed_with_program_unchecked(&[], program)
    }

    /// Invokes the instruction with signer seeds after verifying the `program`
    /// address without checking the borrow state of the accounts.
    ///
    /// # Safety
    ///
    /// This method does not perform any account borrow checking. The borrow
    /// state of the accounts is only checked when an instruction is added to
    /// the batch (e.g., via [`IntoBatch::into_batch`]), not when the batch is
    /// invoked.
    ///
    /// The caller must ensure that none of the batch accounts are borrowed
    /// between adding an instruction to the batch and invoking the batch.
    /// Otherwise, the CPI may modify account data or lamports while a
    /// reference to them is still alive, which is undefined behavior.
    #[inline(always)]
    pub unsafe fn invoke_signed_with_program_unchecked(
        &self,
        signers: &[Signer],
        program: &Address,
    ) -> ProgramResult {
        Program::verify(program)?;
        self.invoke_signed_with_unverified_program_unchecked(signers, program)
    }

    /// Invokes the instruction with `program` without verifying it and
    /// without checking the borrow state of the accounts.
    ///
    /// Use this when `program` has already been verified. Otherwise, prefer
    /// `invoke_with_program_unchecked`.
    ///
    /// # Safety
    ///
    /// This method does not perform any account borrow checking. The borrow
    /// state of the accounts is only checked when an instruction is added to
    /// the batch (e.g., via [`IntoBatch::into_batch`]), not when the batch is
    /// invoked.
    ///
    /// The caller must ensure that none of the batch accounts are borrowed
    /// between adding an instruction to the batch and invoking the batch.
    /// Otherwise, the CPI may modify account data or lamports while a
    /// reference to them is still alive, which is undefined behavior.
    ///
    /// This method does not verify that `program` satisfies
    /// [`TokenInterface::verify`]. The caller must ensure the program address
    /// has already been checked and corresponds to the expected
    /// token program.
    #[inline(always)]
    pub unsafe fn invoke_with_unverified_program_unchecked(
        &self,
        program: &Address,
    ) -> ProgramResult {
        self.invoke_signed_with_unverified_program_unchecked(&[], program)
    }

    /// Invokes the instruction with signer seeds and `program` without
    /// verifying the program address and without checking the borrow state
    /// of the accounts.
    ///
    /// Use this when `program` has already been verified. Otherwise, prefer
    /// `invoke_signed_with_program_unchecked`.
    ///
    /// # Safety
    ///
    /// This method does not perform any account borrow checking. The borrow
    /// state of the accounts is only checked when an instruction is added to
    /// the batch (e.g., via [`IntoBatch::into_batch`]), not when the batch is
    /// invoked.
    ///
    /// The caller must ensure that none of the batch accounts are borrowed
    /// between adding an instruction to the batch and invoking the batch.
    /// Otherwise, the CPI may modify account data or lamports while a
    /// reference to them is still alive, which is undefined behavior.
    ///
    /// This method does not verify that `program` satisfies
    /// [`TokenInterface::verify`]. The caller must ensure the program address
    /// has already been checked and corresponds to the expected
    /// token program.
    #[inline(always)]
    pub unsafe fn invoke_signed_with_unverified_program_unchecked(
        &self,
        signers: &[Signer],
        program: &Address,
    ) -> ProgramResult {
        unsafe {
            invoke_signed_unchecked(
                &InstructionView {
                    program_id: program,
                    accounts: from_raw_parts(
                        self.instruction_accounts.as_ptr() as _,
                        self.instruction_accounts_len,
                    ),
                    data: from_raw_parts(self.data.as_ptr() as _, self.data_len),
                },
                from_raw_parts(self.accounts.as_ptr() as _, self.accounts_len),
                signers,
            );
        }

        Ok(())
    }

    /// Appends an instruction to the batch.
    ///
    /// Each closure receives the remaining (uninitialized) space of the
    /// corresponding buffer and returns the number of elements it wrote:
    ///   - `write_accounts`: the `CpiAccount`s of the instruction.
    ///   - `write_instruction_accounts`: the `InstructionAccount`s of the
    ///     instruction.
    ///   - `write_data`: the instruction data, including the instruction
    ///     discriminator.
    ///
    /// The instruction header (number of accounts and data length) is written
    /// by this method and must not be included in the instruction data.
    ///
    /// If any of the closures returns an error, the error is propagated and
    /// the batch is left unchanged.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    ///   - Each closure initializes the first `n` elements of the slice it
    ///     receives, where `n` is the value it returns, and `n` is not greater
    ///     than the length of that slice.
    ///   - `write_accounts` and `write_instruction_accounts` return the same
    ///     number of accounts, written in the same order.
    ///   - `write_data` does not write more than `255` bytes of instruction
    ///     data.
    ///   - The number of accounts and the length of the instruction data are
    ///     not greater than the length of the corresponding slice passed to the
    ///     closure and do not exceed `255` accounts.
    #[inline(always)]
    pub unsafe fn push(
        &mut self,
        write_accounts: impl FnOnce(
            &mut [MaybeUninit<CpiAccount<'account>>],
        ) -> Result<usize, ProgramError>,
        write_instruction_accounts: impl FnOnce(
            &mut [MaybeUninit<InstructionAccount<'account>>],
        ) -> Result<usize, ProgramError>,
        write_data: impl FnOnce(&mut [MaybeUninit<u8>]) -> Result<usize, ProgramError>,
    ) -> ProgramResult {
        // Ensure that there is enough space for another instruction.
        if self.data_len + IX_HEADER_SIZE > self.data.len() {
            return Err(invalid_argument_error());
        }

        let written_data = write_data(&mut self.data[self.data_len + IX_HEADER_SIZE..])?;

        let written_accounts = write_accounts(&mut self.accounts[self.accounts_len..])?;

        let written_instruction_accounts = write_instruction_accounts(
            &mut self.instruction_accounts[self.instruction_accounts_len..],
        )?;

        // If all writes succeeded, update the lengths and write the instruction
        // header.

        self.accounts_len += written_accounts;
        self.instruction_accounts_len += written_instruction_accounts;

        self.data[self.data_len].write(written_instruction_accounts as u8);
        self.data[self.data_len + 1].write(written_data as u8);
        self.data_len += written_data + IX_HEADER_SIZE;

        Ok(())
    }

    /// Returns the length of the instruction data header for a batch with the
    /// given number of instructions.
    pub const fn header_data_len(instructions_len: usize) -> usize {
        // 1 bytes discriminator + 2 bytes (header) per instruction
        1usize.saturating_add(instructions_len.saturating_mul(IX_HEADER_SIZE))
    }
}

#[cfg(feature = "alloc")]
/// A state object that contains the buffers for a `Batch` instruction.
pub struct BatchState<'account, Program: TokenInterface> {
    /// Container for the instruction data of the batch instruction.
    data: Box<[MaybeUninit<u8>]>,

    /// Container for the instruction accounts of the batch instruction.
    instruction_accounts: Box<[MaybeUninit<InstructionAccount<'account>>]>,

    /// Container for the accounts of the batch instruction.
    accounts: Box<[MaybeUninit<CpiAccount<'account>>]>,

    /// Phantom data for the program.
    _program: PhantomData<Program>,
}

#[cfg(feature = "alloc")]
impl<'account, Program: TokenInterface> BatchState<'account, Program> {
    #[inline(always)]
    pub fn new(accounts_len: usize, data_len: usize) -> Self {
        Self {
            data: Box::new_uninit_slice(data_len),
            instruction_accounts: Box::new_uninit_slice(accounts_len),
            accounts: Box::new_uninit_slice(accounts_len),
            _program: PhantomData,
        }
    }

    #[inline(always)]
    pub fn as_batch<'state>(
        &'state mut self,
    ) -> Result<Batch<'account, 'state, Program>, ProgramError>
    where
        Self: 'account,
    {
        Batch::new(
            self.data.as_mut(),
            self.instruction_accounts.as_mut(),
            self.accounts.as_mut(),
        )
    }
}

/// A trait for instructions that can be consumed directly into a `Batch`.
///
/// Implementations serialize the instruction accounts and data into the
/// batch buffers, typically through [`Batch::push`]. The instruction is
/// executed when the batch is invoked, together with all other instructions
/// in the batch, in a single CPI to the token program.
///
/// # Safety
///
/// [`Batch`] uses the lengths written by implementations to build the CPI
/// without further validation, so implementations must ensure that
/// `into_batch`:
///   - Upholds all the safety requirements of [`Batch::push`] when appending
///     the instruction.
///   - Checks that writable accounts are not borrowed, returning an error
///     otherwise.
pub unsafe trait IntoBatch<Program: TokenInterface> {
    /// Serializes `self` into the provided batch.
    ///
    /// Returns an error if the batch buffers do not have enough space for
    /// the instruction, or if any of the instruction accounts cannot be used
    /// in a CPI (e.g., a writable account is already borrowed).
    fn into_batch<'account, 'state>(
        self,
        batch: &mut Batch<'account, 'state, Program>,
    ) -> ProgramResult
    where
        Self: 'account + 'state;
}
