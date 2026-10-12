use {
    crate::{instructions::invalid_argument_error, write_bytes, UNINIT_BYTE},
    core::{marker::PhantomData, mem::MaybeUninit, slice::from_raw_parts},
    pinocchio_token::{
        instructions::{batch::Batch, CpiWriter},
        TokenInterface,
    },
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_unchecked, CpiAccount},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// The instruction discriminator.
///
/// Hash of `spl_token_metadata_interface:emitter`.
const DISCRIMINATOR: [u8; 8] = [250, 166, 180, 250, 13, 12, 184, 70];

/// Expected number of accounts.
const ACCOUNTS_LEN: usize = 1;

/// Maximum instruction data length:
///   - discriminator (8 bytes)
///   - start (1 or 9 bytes)
///   - end (1 or 9 bytes)
const MAX_DATA_LEN: usize = 26;

/// Emits the `TokenMetadata` entry as return data.
///
/// The emitted bytes follow the `TokenMetadata` layout, sliced by the
/// optional `start` and `end` offsets. Return data can be fetched using
/// `sol_get_return_data`.
///
/// Accounts expected by this instruction:
///
///   0. `[]` The metadata account.
pub struct Emit<'account, Program: TokenInterface> {
    /// The metadata account.
    pub metadata: &'account AccountView,

    /// Start of the range of data to emit.
    pub start: Option<u64>,

    /// End of the range of data to emit.
    pub end: Option<u64>,

    /// Phantom data for the program.
    _program: PhantomData<Program>,
}

impl<'account, Program: TokenInterface> Emit<'account, Program> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: [u8; 8] = DISCRIMINATOR;

    /// Expected number of accounts.
    pub const ACCOUNTS_LEN: usize = ACCOUNTS_LEN;

    /// Maximum instruction data length.
    pub const MAX_DATA_LEN: usize = MAX_DATA_LEN;

    #[inline(always)]
    pub fn new(metadata: &'account AccountView, start: Option<u64>, end: Option<u64>) -> Self {
        Self {
            metadata,
            start,
            end,
            _program: PhantomData,
        }
    }

    /// Invokes the instruction with `Program::ID`.
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_with_unverified_program(&Program::ID)
    }

    /// Invokes the instruction after verifying the `program` address.
    #[inline(always)]
    pub fn invoke_with_program(&self, program: &Address) -> ProgramResult {
        Program::verify(program)?;
        self.invoke_with_unverified_program(program)
    }

    /// Invokes the instruction with `program` without verifying it.
    ///
    /// Use this when `program` has already been verified. Otherwise, prefer
    /// `invoke_with_program`.
    ///
    /// # Important
    ///
    /// This method does not verify that `program` satisfies
    /// [`TokenInterface::verify`]. The caller must ensure the program address
    /// has already been checked and corresponds to the expected
    /// token program.
    #[inline(always)]
    pub fn invoke_with_unverified_program(&self, program: &Address) -> ProgramResult {
        let mut instruction_accounts = [const { MaybeUninit::uninit() }; ACCOUNTS_LEN];
        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts)?;

        let mut accounts = [const { MaybeUninit::uninit() }; ACCOUNTS_LEN];
        let written_accounts = self.write_accounts(&mut accounts)?;

        let mut instruction_data = [UNINIT_BYTE; MAX_DATA_LEN];
        let written_instruction_data = self.write_instruction_data(&mut instruction_data)?;

        unsafe {
            invoke_unchecked(
                &InstructionView {
                    program_id: program,
                    accounts: from_raw_parts(
                        instruction_accounts.as_ptr() as _,
                        written_instruction_accounts,
                    ),
                    data: from_raw_parts(instruction_data.as_ptr() as _, written_instruction_data),
                },
                from_raw_parts(accounts.as_ptr() as _, written_accounts),
            );
        }

        Ok(())
    }
}

impl<Program: TokenInterface> CpiWriter for Emit<'_, Program> {
    #[inline(always)]
    fn write_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_accounts(self.metadata, accounts)
    }

    #[inline(always)]
    fn write_instruction_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_instruction_accounts(self.metadata, accounts)
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        write_instruction_data(self.start, self.end, data)
    }
}

impl<Program: TokenInterface> crate::instructions::IntoBatch<Program> for Emit<'_, Program> {
    #[inline(always)]
    fn into_batch<'account, 'state>(
        self,
        batch: &mut Batch<'account, 'state, Program>,
    ) -> ProgramResult
    where
        Self: 'account + 'state,
    {
        batch.push(
            |accounts| write_accounts(self.metadata, accounts),
            |accounts| write_instruction_accounts(self.metadata, accounts),
            |data| write_instruction_data(self.start, self.end, data),
        )
    }
}

#[inline(always)]
fn write_accounts<'account, 'out>(
    metadata: &'account AccountView,
    accounts: &mut [MaybeUninit<CpiAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    CpiAccount::init_from_account_view(metadata, &mut accounts[0]);

    Ok(ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_accounts<'account, 'out>(
    metadata: &'account AccountView,
    accounts: &mut [MaybeUninit<InstructionAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    accounts[0].write(InstructionAccount::readonly(metadata.address()));

    Ok(ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_data(
    start: Option<u64>,
    end: Option<u64>,
    data: &mut [MaybeUninit<u8>],
) -> Result<usize, ProgramError> {
    let expected_data_len = 10 + start.map_or(0, |_| 8) + end.map_or(0, |_| 8);

    if data.len() < expected_data_len {
        return Err(invalid_argument_error());
    }

    write_bytes(&mut data[0..8], &DISCRIMINATOR);

    let mut offset = 8;

    for value in [start, end] {
        if let Some(value) = value {
            data[offset].write(1);
            write_bytes(&mut data[offset + 1..offset + 9], &value.to_le_bytes());
            offset += 9;
        } else {
            data[offset].write(0);
            offset += 1;
        }
    }

    Ok(expected_data_len)
}
