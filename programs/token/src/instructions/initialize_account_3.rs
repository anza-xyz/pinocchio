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

/// Like [`super::InitializeAccount2`], but does not require the
/// Rent sysvar to be provided
///
/// Accounts expected by this instruction:
///
///   0. `[writable]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
pub struct InitializeAccount3<'account, 'address> {
    /// The account to initialize.
    pub account: &'account AccountView,

    /// The mint this account will be associated with.
    pub mint: &'account AccountView,

    /// The new account's owner/multisignature.
    pub owner: &'address Address,
}

impl<'account, 'address> InitializeAccount3<'account, 'address> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = 18;

    /// Expected number of accounts.
    pub const ACCOUNTS_LEN: usize = 2;

    /// Instruction data length:
    ///   - discriminator (1 byte)
    ///   - owner pubkey (32 bytes)
    pub const DATA_LEN: usize = 33;

    #[inline(always)]
    pub fn new(
        account: &'account AccountView,
        mint: &'account AccountView,
        owner: &'address Address,
    ) -> Self {
        Self {
            account,
            mint,
            owner,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let mut instruction_accounts =
            [UNINIT_INSTRUCTION_ACCOUNT; InitializeAccount3::ACCOUNTS_LEN];
        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts)?;

        let mut accounts = [UNINIT_CPI_ACCOUNT; InitializeAccount3::ACCOUNTS_LEN];
        let written_accounts = self.write_accounts(&mut accounts)?;

        let mut instruction_data = [UNINIT_BYTE; InitializeAccount3::DATA_LEN];
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

impl CpiWriter for InitializeAccount3<'_, '_> {
    #[inline(always)]
    fn write_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_accounts(self.account, self.mint, accounts)
    }

    #[inline(always)]
    fn write_instruction_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_instruction_accounts(self.account, self.mint, accounts)
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        write_instruction_data(self.owner, data)
    }
}

impl super::IntoBatch for InitializeAccount3<'_, '_> {
    #[inline(always)]
    fn into_batch<'batch>(self, batch: &mut super::Batch<'batch>) -> ProgramResult
    where
        Self: 'batch,
    {
        batch.push(
            |accounts| write_accounts(self.account, self.mint, accounts),
            |accounts| write_instruction_accounts(self.account, self.mint, accounts),
            |data| write_instruction_data(self.owner, data),
        )
    }
}

#[inline(always)]
fn write_accounts<'account, 'out>(
    account: &'account AccountView,
    mint: &'account AccountView,
    accounts: &mut [MaybeUninit<CpiAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < InitializeAccount3::ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    if account.is_borrowed() {
        return Err(account_borrow_failed_error());
    }

    CpiAccount::init_from_account_view(account, &mut accounts[0]);

    CpiAccount::init_from_account_view(mint, &mut accounts[1]);

    Ok(InitializeAccount3::ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_accounts<'account, 'out>(
    account: &'account AccountView,
    mint: &'account AccountView,
    accounts: &mut [MaybeUninit<InstructionAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < InitializeAccount3::ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    accounts[0].write(InstructionAccount::writable(account.address()));

    accounts[1].write(InstructionAccount::readonly(mint.address()));

    Ok(InitializeAccount3::ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_data(
    owner: &Address,
    data: &mut [MaybeUninit<u8>],
) -> Result<usize, ProgramError> {
    if data.len() < InitializeAccount3::DATA_LEN {
        return Err(invalid_argument_error());
    }

    data[0].write(InitializeAccount3::DISCRIMINATOR);

    write_bytes(&mut data[1..InitializeAccount3::DATA_LEN], owner.as_array());

    Ok(InitializeAccount3::DATA_LEN)
}
