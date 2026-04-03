use {
    crate::{
        instructions::{account_borrow_failed_error, invalid_argument_error, CpiWriter},
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

    /// Expected number of accounts.
    pub const ACCOUNTS_LEN: usize = 2;

    /// Instruction data length:
    ///   - discriminator (1 byte)
    ///   - decimals (1 byte)
    ///   - mint authority (32 bytes)
    ///   - freeze authority (33 bytes, optional)
    pub const MAX_DATA_LEN: usize = 67;

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
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; InitializeMint::ACCOUNTS_LEN];
        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts)?;

        let mut accounts = [UNINIT_CPI_ACCOUNT; Self::ACCOUNTS_LEN];
        let written_accounts = self.write_accounts(&mut accounts)?;

        let mut instruction_data = [UNINIT_BYTE; Self::MAX_DATA_LEN];
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
    fn write_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_accounts(self.mint, self.rent_sysvar, accounts)
    }

    #[inline(always)]
    fn write_instruction_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
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

impl super::IntoBatch for InitializeMint<'_, '_> {
    #[inline(always)]
    fn into_batch<'account, 'state>(
        self,
        batch: &mut super::Batch<'account, 'state>,
    ) -> ProgramResult
    where
        Self: 'account + 'state,
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
    if accounts.len() < InitializeMint::ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    if mint.is_borrowed() {
        return Err(account_borrow_failed_error());
    }

    CpiAccount::init_from_account_view(mint, &mut accounts[0]);

    CpiAccount::init_from_account_view(rent_sysvar, &mut accounts[1]);

    Ok(InitializeMint::ACCOUNTS_LEN)
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
    if accounts.len() < InitializeMint::ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    accounts[0].write(InstructionAccount::writable(mint.address()));

    accounts[1].write(InstructionAccount::readonly(rent_sysvar.address()));

    Ok(InitializeMint::ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_data(
    decimals: u8,
    mint_authority: &Address,
    freeze_authority: Option<&Address>,
    data: &mut [MaybeUninit<u8>],
) -> Result<usize, ProgramError> {
    if data.len() < InitializeMint::MAX_DATA_LEN {
        return Err(invalid_argument_error());
    }

    data[0].write(InitializeMint::DISCRIMINATOR);

    data[1].write(decimals);

    write_bytes(&mut data[2..34], mint_authority.as_array());

    if let Some(freeze_authority) = freeze_authority {
        data[34].write(1);

        write_bytes(
            &mut data[35..InitializeMint::MAX_DATA_LEN],
            freeze_authority.as_array(),
        );

        Ok(InitializeMint::MAX_DATA_LEN)
    } else {
        data[34].write(0);

        Ok(35)
    }
}
