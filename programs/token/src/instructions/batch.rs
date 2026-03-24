use {
    crate::instructions::{invalid_argument_error, CpiWriter},
    alloc::boxed::Box,
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_instruction_view::{
        cpi::{invoke_signed_unchecked, CpiAccount, Signer, MAX_CPI_ACCOUNTS},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Maximum CPI instruction data size.
const MAX_CPI_INSTRUCTION_DATA_LEN: usize = 10 * 1024;

/// The size of the batch instruction header.
///
/// The header of each instruction consists of two `u8` values:
///   - number of the accounts
///   - length of the instruction data
const IX_HEADER_SIZE: usize = 2;

/// A collection of instructions that can be serialized into a token `Batch`
/// instruction.
pub struct Batch<'a> {
    /// The instruction data for the batch instruction. The first byte is
    /// reserved for the batch instruction discriminator, and each
    /// instruction's data is prefixed with a byte indicating the number of
    /// instruction accounts and a byte indicating the length of the
    /// instruction data.
    data: Box<[MaybeUninit<u8>]>,

    /// The instruction accounts for the batch instruction.
    instruction_accounts: Box<[MaybeUninit<InstructionAccount<'a>>]>,

    /// The accounts for the batch instruction.
    accounts: Box<[MaybeUninit<CpiAccount<'a>>]>,

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

    #[inline(always)]
    pub fn new() -> Self {
        let mut data: Box<[MaybeUninit<u8>]> = Box::new_uninit_slice(MAX_CPI_INSTRUCTION_DATA_LEN);
        // The first byte of the instruction data is reserved for the batch instruction
        // discriminator.
        data[0].write(Self::DISCRIMINATOR);

        Self {
            data,
            instruction_accounts: Box::new_uninit_slice(MAX_CPI_ACCOUNTS),
            accounts: Box::new_uninit_slice(MAX_CPI_ACCOUNTS),
            data_len: 1,
            accounts_len: 0,
            instruction_accounts_len: 0,
        }
    }

    #[inline(always)]
    pub fn push<T: Batchable + ?Sized>(&mut self, instruction: &'a T) -> ProgramResult {
        self.push_encoded(
            |accounts| instruction.write_accounts(accounts),
            |instruction_accounts| instruction.write_instruction_accounts(instruction_accounts),
            |data| instruction.write_instruction_data(data),
        )
    }

    #[inline(always)]
    pub fn append(&mut self, instructions: &[&'a dyn Batchable]) -> ProgramResult {
        for instruction in instructions {
            self.push(*instruction)?;
        }

        Ok(())
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
                from_raw_parts(
                    self.accounts.as_ptr() as *const CpiAccount,
                    self.accounts_len,
                ),
                signers,
            );
        }

        Ok(())
    }

    #[inline(always)]
    pub(crate) fn push_encoded(
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

        // If all writres succeeded, update the lengths and write the instruction header.

        self.accounts_len += written_accounts;
        self.instruction_accounts_len += written_instruction_accounts;

        self.data[self.data_len].write(written_instruction_accounts as u8);
        self.data[self.data_len + 1].write(written_data as u8);
        self.data_len += written_data + IX_HEADER_SIZE;

        Ok(())
    }
}

impl Default for Batch<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// A trait for instructions that can be consumed directly into a `Batch`.
pub trait IntoBatch: sealed::Sealed {
    /// Serializes `self` into the provided batch.
    fn into_batch<'batch>(self, batch: &mut Batch<'batch>) -> ProgramResult
    where
        Self: 'batch;
}

/// Marker trait for instructions that can be used in a `Batch`.
///
/// This trait is automatically implemented for all types that
/// implement `CpiWriter`.
pub trait Batchable: CpiWriter + sealed::Sealed {}

/// Implement `Sealed` for all types that implement `CpiWriter`.
impl<T: CpiWriter> sealed::Sealed for T {}

/// Implement `Batchable` for all types that implement `CpiWriter`
/// and are sealed.
impl<T: CpiWriter + sealed::Sealed + ?Sized> Batchable for T {}

/// A module only accessible within this crate that contains the
/// `Sealed` trait.
pub(crate) mod sealed {
    /// A sealed trait that prevents external implementations of the
    /// `Batchable` trait.
    pub trait Sealed {}
}
