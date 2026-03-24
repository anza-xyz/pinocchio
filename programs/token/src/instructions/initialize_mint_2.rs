use {
    crate::{
        instructions::{invalid_argument_error, writable_cpi_account, CpiWriter},
        write_bytes, UNINIT_BYTE, UNINIT_CPI_ACCOUNT, UNINIT_INSTRUCTION_ACCOUNT,
    },
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_unchecked, CpiAccount},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Expected number of accounts.
const ACCOUNTS_LEN: usize = 1;

/// Instruction data length:
///   - discriminator (1 byte)
///   - decimals (1 byte)
///   - mint authority (32 bytes)
///   - freeze authority (33 bytes, optional)
const MAX_DATA_LEN: usize = 67;

/// Like [`super::InitializeMint`], but does not require the Rent
/// sysvar to be provided
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint to initialize.
pub struct InitializeMint2<'account> {
    /// The mint to initialize.
    pub mint: &'account AccountView,

    /// The number of base 10 digits to the right of the decimal place.
    pub decimals: u8,

    /// The authority/multisignature to mint tokens.
    pub mint_authority: &'account Address,

    /// The freeze authority/multisignature of the mint.
    pub freeze_authority: Option<&'account Address>,
}

impl InitializeMint2<'_> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = 20;

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

impl CpiWriter for InitializeMint2<'_> {
    #[inline(always)]
    fn write_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        'source: 'cpi,
    {
        write_accounts(self.mint, accounts)
    }

    #[inline(always)]
    fn write_instruction_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        'source: 'cpi,
    {
        write_instruction_accounts(self.mint, accounts)
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        write_instruction_data(
            self.decimals,
            self.mint_authority,
            self.freeze_authority,
            data,
        )
    }
}

#[cfg(feature = "batch")]
impl super::IntoBatch for InitializeMint2<'_> {
    #[inline(always)]
    fn into_batch<'batch>(self, batch: &mut super::Batch<'batch>) -> ProgramResult
    where
        Self: 'batch,
    {
        batch.push(
            |accounts| write_accounts(self.mint, accounts),
            |accounts| write_instruction_accounts(self.mint, accounts),
            |data| {
                write_instruction_data(
                    self.decimals,
                    self.mint_authority,
                    self.freeze_authority,
                    data,
                )
            },
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

    accounts[0].write(writable_cpi_account(mint)?);

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

    accounts[0].write(InstructionAccount::writable(mint.address()));

    Ok(ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_data(
    decimals: u8,
    mint_authority: &Address,
    freeze_authority: Option<&Address>,
    data: &mut [MaybeUninit<u8>],
) -> Result<usize, ProgramError> {
    if data.len() < MAX_DATA_LEN {
        return Err(invalid_argument_error());
    }

    data[0].write(InitializeMint2::DISCRIMINATOR);

    data[1].write(decimals);

    write_bytes(&mut data[2..34], mint_authority.as_array());

    if let Some(freeze_auth) = freeze_authority {
        data[34].write(1);

        write_bytes(&mut data[35..MAX_DATA_LEN], freeze_auth.as_array());

        Ok(MAX_DATA_LEN)
    } else {
        data[34].write(0);

        Ok(35)
    }
}
