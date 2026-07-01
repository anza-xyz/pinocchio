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
const DISCRIMINATOR: u8 = 24;

/// Expected number of accounts.
const ACCOUNTS_LEN: usize = 1;

/// Instruction data length:
///   - discriminator (1 byte)
///   - amount (variable, up to 254 bytes)
const MAX_DATA_LEN: usize = 255;

/// Convert a `UiAmount` of tokens to a little-endian `u64` raw Amount,
/// using the given mint. In this version of the program, the mint can
/// only specify the number of decimals.
///
/// Return data can be fetched using `sol_get_return_data` and deserializing
/// the return data as a little-endian `u64`.
///
/// Accounts expected by this instruction:
///
///   0. `[]` The mint to calculate for.
pub struct UiAmountToAmount<'account, 'amount, Program: TokenProgram> {
    /// The mint to calculate for.
    pub mint: &'account AccountView,

    /// The `ui_amount` of tokens to reformat.
    pub amount: &'amount str,

    _program: PhantomData<Program>,
}

impl<'account, 'amount, Program: TokenProgram> UiAmountToAmount<'account, 'amount, Program> {
    #[inline(always)]
    pub fn new(mint: &'account AccountView, amount: &'amount str) -> Self {
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

        let mut instruction_data = [UNINIT_BYTE; MAX_DATA_LEN];
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

impl<Program: TokenProgram> super::IntoBatch<Program> for UiAmountToAmount<'_, '_, Program> {
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

impl<Program: TokenProgram> CpiWriter for UiAmountToAmount<'_, '_, Program> {
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
    amount: &str,
    data: &mut [MaybeUninit<u8>],
) -> Result<usize, ProgramError> {
    let expected_data_len = 1 + amount.len();

    if expected_data_len > MAX_DATA_LEN || data.len() < expected_data_len {
        return Err(invalid_argument_error());
    }

    data[0].write(DISCRIMINATOR);

    write_bytes(&mut data[1..expected_data_len], amount.as_bytes());

    Ok(expected_data_len)
}
