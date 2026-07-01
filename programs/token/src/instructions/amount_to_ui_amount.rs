use {
    crate::{
        instructions::{invalid_argument_error, CpiWriter, TokenProgram},
        write_bytes, UNINIT_BYTE, UNINIT_CPI_ACCOUNT, UNINIT_INSTRUCTION_ACCOUNT,
    },
    core::{marker::PhantomData, mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_unchecked, CpiAccount},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// The instruction discriminator.
const DISCRIMINATOR: u8 = 23;

/// Expected number of accounts.
const ACCOUNTS_LEN: usize = 1;

/// Instruction data length:
///   - discriminator (1 byte)
///   - amount (8 bytes)
const DATA_LEN: usize = 9;

/// Convert an Amount of tokens to a `UiAmount` string, using the given
/// mint.
///
/// Fails on an invalid mint.
///
/// Return data can be fetched using `sol_get_return_data` and deserialized
/// with `String::from_utf8`.
///
/// Accounts expected by this instruction:
///
///   0. `[]` The mint to calculate for.
pub struct AmountToUiAmount<'account, Program: TokenProgram> {
    /// The mint to calculate for.
    pub mint: &'account AccountView,

    /// The amount of tokens to reformat.
    pub amount: u64,

    _program: PhantomData<Program>,
}

impl<'account, Program: TokenProgram> AmountToUiAmount<'account, Program> {
    #[inline(always)]
    pub fn new(mint: &'account AccountView, amount: u64) -> Self {
        Self {
            mint,
            amount,
            _program: PhantomData,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
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
                    program_id: &Program::ID,
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

impl<Program: TokenProgram> CpiWriter for AmountToUiAmount<'_, Program> {
    #[inline(always)]
    fn write_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_accounts(self.mint, accounts)
    }

    #[inline(always)]
    fn write_instruction_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_instruction_accounts(self.mint, accounts)
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        write_instruction_data(self.amount, data)
    }
}

impl<Program: TokenProgram> super::IntoBatch<Program> for AmountToUiAmount<'_, Program> {
    #[inline(always)]
    fn into_batch<'account, 'state>(
        self,
        batch: &mut super::Batch<'account, 'state, Program>,
    ) -> ProgramResult
    where
        Self: 'account + 'state,
    {
        batch.push(
            |accounts| write_accounts(self.mint, accounts),
            |accounts| write_instruction_accounts(self.mint, accounts),
            |data| write_instruction_data(self.amount, data),
        )
    }
}

#[inline(always)]
fn write_accounts<'account, 'out>(
    mint: &'account AccountView,
    accounts: &mut [MaybeUninit<CpiAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    CpiAccount::init_from_account_view(mint, &mut accounts[0]);

    Ok(ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_accounts<'account, 'out>(
    mint: &'account AccountView,
    accounts: &mut [MaybeUninit<InstructionAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    accounts[0].write(InstructionAccount::readonly(mint.address()));

    Ok(ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_data(
    amount: u64,
    data: &mut [MaybeUninit<u8>],
) -> Result<usize, ProgramError> {
    if data.len() < DATA_LEN {
        return Err(invalid_argument_error());
    }

    data[0].write(DISCRIMINATOR);

    write_bytes(&mut data[1..DATA_LEN], &amount.to_le_bytes());

    Ok(DATA_LEN)
}
