use {
    crate::{
        instructions::{
            account_borrow_failed_error, invalid_argument_error, CpiWriter, UNINIT_BYTE,
            UNINIT_CPI_ACCOUNT, UNINIT_INSTRUCTION_ACCOUNT,
        },
        TokenInterface,
    },
    core::{marker::PhantomData, mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_unchecked, CpiAccount},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// The instruction discriminator.
const DISCRIMINATOR: u8 = 22;

/// Expected number of accounts.
const ACCOUNTS_LEN: usize = 1;

/// Instruction data length:
///   - discriminator (1 byte)
const DATA_LEN: usize = 1;

/// Initialize the Immutable Owner extension for the given token account
///
/// Fails if the account has already been initialized, so must be called
/// before `InitializeAccount`.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The account to initialize.
pub struct InitializeImmutableOwner<'account, Program: TokenInterface> {
    /// The account to initialize.
    pub account: &'account AccountView,

    /// Phantom data for the program.
    _program: PhantomData<Program>,
}

impl<'account, Program: TokenInterface> InitializeImmutableOwner<'account, Program> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = DISCRIMINATOR;

    /// Expected number of accounts.
    pub const ACCOUNTS_LEN: usize = ACCOUNTS_LEN;

    /// Instruction data length.
    pub const DATA_LEN: usize = DATA_LEN;

    #[inline(always)]
    pub fn new(account: &'account AccountView) -> Self {
        Self {
            account,
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
    /// [`TokenProgram::verify`]. The caller must ensure the program address
    /// has already been checked and corresponds to the expected
    /// token program.
    #[inline(always)]
    pub fn invoke_with_unverified_program(&self, program: &Address) -> ProgramResult {
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; ACCOUNTS_LEN];
        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts)?;

        let mut accounts = [UNINIT_CPI_ACCOUNT; ACCOUNTS_LEN];
        let written_accounts = self.write_accounts(&mut accounts)?;

        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];
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

impl<Program: TokenInterface> CpiWriter for InitializeImmutableOwner<'_, Program> {
    #[inline(always)]
    fn write_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_accounts(self.account, accounts)
    }

    #[inline(always)]
    fn write_instruction_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_instruction_accounts(self.account, accounts)
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        write_instruction_data(data)
    }
}

impl<Program: TokenInterface> super::batch::IntoBatch<Program>
    for InitializeImmutableOwner<'_, Program>
{
    #[inline(always)]
    fn into_batch<'account, 'state>(
        self,
        batch: &mut super::batch::Batch<'account, 'state, Program>,
    ) -> ProgramResult
    where
        Self: 'account + 'state,
    {
        batch.push(
            |accounts| write_accounts(self.account, accounts),
            |accounts| write_instruction_accounts(self.account, accounts),
            write_instruction_data,
        )
    }
}

#[inline(always)]
fn write_accounts<'account, 'out>(
    account: &'account AccountView,
    accounts: &mut [MaybeUninit<CpiAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    if account.is_borrowed() {
        return Err(account_borrow_failed_error());
    }

    CpiAccount::init_from_account_view(account, &mut accounts[0]);

    Ok(ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_accounts<'account, 'out>(
    account: &'account AccountView,
    accounts: &mut [MaybeUninit<InstructionAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    accounts[0].write(InstructionAccount::writable(account.address()));

    Ok(ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_data(data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
    if data.len() < DATA_LEN {
        return Err(invalid_argument_error());
    }

    data[0].write(DISCRIMINATOR);

    Ok(DATA_LEN)
}
