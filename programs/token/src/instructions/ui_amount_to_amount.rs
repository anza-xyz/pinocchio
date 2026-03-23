use {
    crate::{
        instructions::{cpi_account, invalid_argument_error, CpiWriter},
        write_bytes, UNINIT_BYTE, UNINIT_CPI_ACCOUNT, UNINIT_INSTRUCTION_ACCOUNT,
    },
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_unchecked, CpiAccount},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Expected number of accounts.
const ACCOUNTS_LEN: usize = 1;

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
pub struct UiAmountToAmount<'account, 'amount, const LENGTH: usize> {
    /// The mint to calculate for.
    pub mint: &'account AccountView,

    /// The `ui_amount` of tokens to reformat.
    pub amount: &'amount str,
}

impl<const LENGTH: usize> UiAmountToAmount<'_, '_, LENGTH> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = 24;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; ACCOUNTS_LEN];
        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts)?;

        let mut accounts = [UNINIT_CPI_ACCOUNT; ACCOUNTS_LEN];
        let written_accounts = self.write_accounts(&mut accounts)?;

        let mut instruction_data = [UNINIT_BYTE; LENGTH];
        let written_instruction_data = self.write_instruction_data(&mut instruction_data)?;

        unsafe {
            invoke_unchecked(
                &InstructionView {
                    program_id: &crate::ID,
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

impl<const LENGTH: usize> CpiWriter for UiAmountToAmount<'_, '_, LENGTH> {
    #[inline(always)]
    fn write_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        'source: 'cpi,
    {
        if accounts.len() < ACCOUNTS_LEN {
            return Err(invalid_argument_error());
        }

        accounts[0].write(cpi_account(self.mint)?);

        Ok(ACCOUNTS_LEN)
    }

    #[inline(always)]
    fn write_instruction_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        'source: 'cpi,
    {
        if accounts.len() < ACCOUNTS_LEN {
            return Err(invalid_argument_error());
        }

        accounts[0].write(InstructionAccount::readonly(self.mint.address()));

        Ok(ACCOUNTS_LEN)
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        let expected_data_len = 1 + self.amount.len();

        if data.len() < expected_data_len {
            return Err(invalid_argument_error());
        }

        data[0].write(Self::DISCRIMINATOR);

        write_bytes(&mut data[1..expected_data_len], self.amount.as_bytes());

        Ok(expected_data_len)
    }
}
