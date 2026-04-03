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

/// Like [`super::InitializeAccount`], but the owner pubkey is
/// passed via instruction data rather than the accounts list. This
/// variant may be preferable when using Cross Program Invocation from
/// an instruction that does not need the owner's `AccountInfo`
/// otherwise.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
///   2. `[]` Rent sysvar.
pub struct InitializeAccount2<'account> {
    /// The account to initialize.
    pub account: &'account AccountView,

    /// The mint this account will be associated with.
    pub mint: &'account AccountView,

    /// Rent sysvar.
    pub rent_sysvar: &'account AccountView,

    /// The new account's owner/multisignature.
    pub owner: &'account Address,
}

impl<'account> InitializeAccount2<'account> {
    pub const DISCRIMINATOR: u8 = 16;

    /// Expected number of accounts.
    pub const ACCOUNTS_LEN: usize = 3;

    /// Instruction data length:
    ///   - discriminator (1 byte)
    ///   - owner pubkey (32 bytes)
    pub const DATA_LEN: usize = 33;

    #[inline(always)]
    pub fn new(
        account: &'account AccountView,
        mint: &'account AccountView,
        rent_sysvar: &'account AccountView,
        owner: &'account Address,
    ) -> Self {
        Self {
            account,
            mint,
            rent_sysvar,
            owner,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let mut instruction_accounts =
            [UNINIT_INSTRUCTION_ACCOUNT; InitializeAccount2::ACCOUNTS_LEN];
        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts)?;

        let mut accounts = [UNINIT_CPI_ACCOUNT; Self::ACCOUNTS_LEN];
        let written_accounts = self.write_accounts(&mut accounts)?;

        let mut instruction_data = [UNINIT_BYTE; Self::DATA_LEN];
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
                from_raw_parts(accounts.as_ptr() as *const CpiAccount, written_accounts),
            );
        }

        Ok(())
    }
}

impl CpiWriter for InitializeAccount2<'_> {
    #[inline(always)]
    fn write_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_accounts(self.account, self.mint, self.rent_sysvar, accounts)
    }

    #[inline(always)]
    fn write_instruction_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_instruction_accounts(self.account, self.mint, self.rent_sysvar, accounts)
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        write_instruction_data(self.owner, data)
    }
}

impl super::IntoBatch for InitializeAccount2<'_> {
    #[inline(always)]
    fn into_batch<'account, 'state>(
        self,
        batch: &mut super::Batch<'account, 'state>,
    ) -> ProgramResult
    where
        Self: 'account + 'state,
    {
        batch.push(
            |accounts| write_accounts(self.account, self.mint, self.rent_sysvar, accounts),
            |accounts| {
                write_instruction_accounts(self.account, self.mint, self.rent_sysvar, accounts)
            },
            |data| write_instruction_data(self.owner, data),
        )
    }
}

#[inline(always)]
fn write_accounts<'account, 'out>(
    account: &'account AccountView,
    mint: &'account AccountView,
    rent_sysvar: &'account AccountView,
    accounts: &mut [MaybeUninit<CpiAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < InitializeAccount2::ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    if account.is_borrowed() {
        return Err(account_borrow_failed_error());
    }

    CpiAccount::init_from_account_view(account, &mut accounts[0]);

    CpiAccount::init_from_account_view(mint, &mut accounts[1]);

    CpiAccount::init_from_account_view(rent_sysvar, &mut accounts[2]);

    Ok(InitializeAccount2::ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_accounts<'account, 'out>(
    account: &'account AccountView,
    mint: &'account AccountView,
    rent_sysvar: &'account AccountView,
    accounts: &mut [MaybeUninit<InstructionAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < InitializeAccount2::ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    accounts[0].write(InstructionAccount::writable(account.address()));

    accounts[1].write(InstructionAccount::readonly(mint.address()));

    accounts[2].write(InstructionAccount::readonly(rent_sysvar.address()));

    Ok(InitializeAccount2::ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_data(
    owner: &Address,
    data: &mut [MaybeUninit<u8>],
) -> Result<usize, ProgramError> {
    if data.len() < InitializeAccount2::DATA_LEN {
        return Err(invalid_argument_error());
    }

    data[0].write(InitializeAccount2::DISCRIMINATOR);

    write_bytes(&mut data[1..InitializeAccount2::DATA_LEN], owner.as_array());

    Ok(InitializeAccount2::DATA_LEN)
}
