use {
    crate::instructions::CpiWriter,
    alloc::boxed::Box,
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_instruction_view::{
        cpi::{invoke_signed_unchecked, CpiAccount, Signer, MAX_CPI_ACCOUNTS},
        InstructionAccount, InstructionView,
    },
    solana_program_error::ProgramResult,
};

/// Maximum CPI instruction data size.
const MAX_CPI_INSTRUCTION_DATA_LEN: usize = 10 * 1024;

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
        self.accounts_len += instruction.write_accounts(&mut self.accounts[self.accounts_len..])?;

        let instruction_accounts_len = instruction.write_instruction_accounts(
            &mut self.instruction_accounts[self.instruction_accounts_len..],
        )?;

        let data_len = instruction.write_instruction_data(&mut self.data[self.data_len + 2..])?;

        self.data[self.data_len].write(instruction_accounts_len as u8);
        self.data[self.data_len + 1].write(data_len as u8);

        self.instruction_accounts_len += instruction_accounts_len;
        self.data_len += data_len + 2;

        Ok(())
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
}

impl Default for Batch<'_> {
    fn default() -> Self {
        Self::new()
    }
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
