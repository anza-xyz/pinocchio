#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use {
    crate::{instructions::invalid_argument_error, TokenProgram},
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
pub struct Batch<'account, 'state, Program: TokenProgram> {
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

impl<'account, 'state, Program: TokenProgram> Batch<'account, 'state, Program>
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

    /// Invokes the instruction with `Program::ID`.
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_with_unverified_program(&Program::ID)
    }

    /// Invokes the instruction with `Program::ID` and signer seeds.
    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        self.invoke_signed_with_unverified_program(signers, &Program::ID)
    }

    /// Invokes the instruction after verifying the `program` address.
    #[inline(always)]
    pub fn invoke_with_program(&self, program: &Address) -> ProgramResult {
        self.invoke_signed_with_program(&[], program)
    }

    /// Invokes the instruction with signer seeds after verifying the `program`
    /// address.
    #[inline(always)]
    pub fn invoke_signed_with_program(
        &self,
        signers: &[Signer],
        program: &Address,
    ) -> ProgramResult {
        Program::verify(program)?;
        self.invoke_signed_with_unverified_program(signers, program)
    }

    /// Invokes the instruction with `program` without verifying it.
    ///
    /// Use this when `program` has already been verified. Otherwise, prefer
    /// `invoke_with_program`.
    ///
    /// # Important
    ///
    /// This method does not verify that `program` satisfies
    /// [`TokenProgram::verify`]. The caller must ensure the program address
    /// has already been checked and corresponds to the expected
    /// token program.
    #[inline(always)]
    pub fn invoke_with_unverified_program(&self, program: &Address) -> ProgramResult {
        self.invoke_signed_with_unverified_program(&[], program)
    }

    /// Invokes the instruction with signer seeds and `program` without
    /// verifying the program address.
    ///
    /// Use this when `program` has already been verified. Otherwise, prefer
    /// `invoke_signed_with_program`.
    ///
    /// # Important
    ///
    /// This method does not verify that `program` satisfies
    /// [`TokenProgram::verify`]. The caller must ensure the program address
    /// has already been checked and corresponds to the expected
    /// token program.
    #[inline(always)]
    pub fn invoke_signed_with_unverified_program(
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

    #[inline(always)]
    pub fn push(
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
pub struct BatchState<'account, Program: TokenProgram> {
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
impl<'account, Program: TokenProgram> BatchState<'account, Program> {
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
pub trait IntoBatch<Program: TokenProgram> {
    /// Serializes `self` into the provided batch.
    fn into_batch<'account, 'state>(
        self,
        batch: &mut Batch<'account, 'state, Program>,
    ) -> ProgramResult
    where
        Self: 'account + 'state;
}
