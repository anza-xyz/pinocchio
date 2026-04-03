#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use {
    crate::instructions::{invalid_argument_error, CpiWriter},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_instruction_view::{
        cpi::{invoke_signed_unchecked, CpiAccount, Signer, MAX_CPI_ACCOUNTS},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// The size of the batch instruction header.
///
/// The header of each instruction consists of two `u8` values:
///   - number of the accounts
///   - length of the instruction data
const IX_HEADER_SIZE: usize = 2;

/// A collection of instructions that can be serialized into a token `Batch`
/// instruction.
pub struct Batch<'a> {
    /// The instruction data for the batch instruction.
    ///
    /// The first byte is reserved for the batch instruction discriminator,
    /// and each instruction's data is prefixed with a byte indicating the
    /// number of instruction accounts and a byte indicating the length of
    /// the instruction data.
    data: &'a mut [MaybeUninit<u8>],

    /// The instruction accounts for the batch instruction.
    instruction_accounts: &'a mut [MaybeUninit<InstructionAccount<'a>>],

    /// The accounts for the batch instruction.
    accounts: &'a mut [MaybeUninit<CpiAccount<'a>>],

    /// The current length of the instruction data.
    data_len: usize,

    /// The current length of the accounts.
    accounts_len: usize,

    /// The current length of the instruction accounts.    
    instruction_accounts_len: usize,
}

impl<'a> Batch<'a> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = 255;

    /// The maximum instruction data buffer length required for a batch.
    pub const MAX_DATA_LEN: usize = 10 * 1024;

    /// The maximum account buffer length required for a batch.
    pub const MAX_ACCOUNTS_LEN: usize = MAX_CPI_ACCOUNTS;

    /// Creates a new `Batch` with the provided buffers.
    #[inline(always)]
    pub fn new(
        data: &'a mut [MaybeUninit<u8>],
        instruction_accounts: &'a mut [MaybeUninit<InstructionAccount<'a>>],
        accounts: &'a mut [MaybeUninit<CpiAccount<'a>>],
    ) -> Result<Self, ProgramError> {
        if data.is_empty() {
            return Err(invalid_argument_error());
        }

        // The first byte of the instruction data is reserved for
        // the batch discriminator.
        data[0].write(Self::DISCRIMINATOR);

        Ok(Self {
            data,
            instruction_accounts,
            accounts,
            data_len: 1,
            accounts_len: 0,
            instruction_accounts_len: 0,
        })
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        unsafe {
            invoke_signed_unchecked(
                &InstructionView {
                    program_id: &crate::ID,
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
    pub(crate) fn push(
        &mut self,
        write_accounts: impl FnOnce(&mut [MaybeUninit<CpiAccount<'a>>]) -> Result<usize, ProgramError>,
        write_instruction_accounts: impl FnOnce(
            &mut [MaybeUninit<InstructionAccount<'a>>],
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
}

#[cfg(feature = "alloc")]
/// A state object that contains the buffers for a `Batch` instruction.
pub struct BatchState<'account> {
    /// Container for the instruction data of the batch instruction.
    data: Box<[MaybeUninit<u8>]>,

    /// Container for the instruction accounts of the batch instruction.
    instruction_accounts: Box<[MaybeUninit<InstructionAccount<'account>>]>,

    /// Container for the accounts of the batch instruction.
    accounts: Box<[MaybeUninit<CpiAccount<'account>>]>,
}

#[cfg(feature = "alloc")]
impl<'account> BatchState<'account> {
    #[inline(always)]
    pub fn new(accounts_len: usize, data_len: usize) -> Self {
        Self {
            data: Box::new_uninit_slice(data_len),
            instruction_accounts: Box::new_uninit_slice(accounts_len),
            accounts: Box::new_uninit_slice(accounts_len),
        }
    }

    #[inline(always)]
    pub fn as_batch(&'account mut self) -> Result<Batch<'account>, ProgramError> {
        Batch::new(
            self.data.as_mut(),
            self.instruction_accounts.as_mut(),
            self.accounts.as_mut(),
        )
    }
}

/// A trait for instructions that can be consumed directly into a `Batch`.
pub trait IntoBatch: sealed::Sealed {
    /// Serializes `self` into the provided batch.
    fn into_batch<'batch>(self, batch: &mut Batch<'batch>) -> ProgramResult
    where
        Self: 'batch;
}

/// Implement `Sealed` for all types that implement `CpiWriter`.
impl<T: CpiWriter> sealed::Sealed for T {}

/// A module only accessible within this crate that contains the
/// `Sealed` trait.
pub(crate) mod sealed {
    /// A sealed trait that prevents external implementations of the
    /// `IntoBatch` trait.
    pub trait Sealed {}
}
