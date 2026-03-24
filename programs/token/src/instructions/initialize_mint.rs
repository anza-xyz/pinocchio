use {
    crate::{
        instructions::{cpi_account, invalid_argument_error, writable_cpi_account, CpiWriter},
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
const ACCOUNTS_LEN: usize = 2;

/// Instruction data length:
///   - discriminator (1 byte)
///   - decimals (1 byte)
///   - mint authority (32 bytes)
///   - freeze authority (33 bytes, optional)
const MAX_DATA_LEN: usize = 67;

/// Initializes a new mint and optionally deposits all the newly minted
/// tokens in an account.
///
/// The `InitializeMint` instruction requires no signers and MUST be
/// included within the same Transaction as the system program's
/// `CreateAccount` instruction that creates the account being initialized.
/// Otherwise another party can acquire ownership of the uninitialized
/// account.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint to initialize.
///   1. `[]` Rent sysvar.
pub struct InitializeMint<'account, 'address> {
    /// The mint to initialize.
    pub mint: &'account AccountView,

    /// Rent sysvar.
    pub rent_sysvar: &'account AccountView,

    /// The number of base 10 digits to the right of the decimal place.
    pub decimals: u8,

    /// The authority/multisignature to mint tokens.
    pub mint_authority: &'address Address,

    /// The freeze authority/multisignature of the mint.
    pub freeze_authority: Option<&'address Address>,
}

impl<'account, 'address> InitializeMint<'account, 'address> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn new(
        mint: &'account AccountView,
        rent_sysvar: &'account AccountView,
        decimals: u8,
        mint_authority: &'address Address,
        freeze_authority: Option<&'address Address>,
    ) -> Self {
        Self {
            mint,
            rent_sysvar,
            decimals,
            mint_authority,
            freeze_authority,
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

impl CpiWriter for InitializeMint<'_, '_> {
    #[inline(always)]
    fn write_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        'source: 'cpi,
    {
        write_accounts(self.mint, self.rent_sysvar, accounts)
    }

    #[inline(always)]
    fn write_instruction_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        'source: 'cpi,
    {
        write_instruction_accounts(self.mint, self.rent_sysvar, accounts)
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
impl super::IntoBatch for InitializeMint<'_, '_> {
    #[inline(always)]
    fn into_batch<'batch>(self, batch: &mut super::Batch<'batch>) -> ProgramResult
    where
        Self: 'batch,
    {
        batch.push(
            |accounts| write_accounts(self.mint, self.rent_sysvar, accounts),
            |accounts| write_instruction_accounts(self.mint, self.rent_sysvar, accounts),
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
    rent_sysvar: &'account AccountView,
    accounts: &mut [MaybeUninit<CpiAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    accounts[0].write(writable_cpi_account(mint)?);

    accounts[1].write(cpi_account(rent_sysvar)?);

    Ok(ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_accounts<'account, 'out>(
    mint: &'account AccountView,
    rent_sysvar: &'account AccountView,
    accounts: &mut [MaybeUninit<InstructionAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    accounts[0].write(InstructionAccount::writable(mint.address()));

    accounts[1].write(InstructionAccount::readonly(rent_sysvar.address()));

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

    data[0].write(InitializeMint::DISCRIMINATOR);

    data[1].write(decimals);

    write_bytes(&mut data[2..34], mint_authority.as_array());

    if let Some(freeze_authority) = freeze_authority {
        data[34].write(1);

        write_bytes(&mut data[35..MAX_DATA_LEN], freeze_authority.as_array());

        Ok(MAX_DATA_LEN)
    } else {
        data[34].write(0);

        Ok(35)
    }
}
