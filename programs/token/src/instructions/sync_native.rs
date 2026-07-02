use {
    crate::{
        instructions::{
            account_borrow_failed_error,
            invalid_argument_error, CpiWriter, UNINIT_BYTE, UNINIT_CPI_ACCOUNT,
            UNINIT_INSTRUCTION_ACCOUNT,
        },
        TokenProgram,
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
const DISCRIMINATOR: u8 = 17;

/// Maximum number of accounts expected by this instruction.
///
/// The required number of accounts will depend whether the instruction uses
/// the runtime Rent sysvar or the Rent sysvar account.
const MAX_ACCOUNTS_LEN: usize = 2;

/// Instruction data length:
///   - discriminator (1 byte)
const DATA_LEN: usize = 1;

/// Given a wrapped / native token account (a token account containing SOL)
/// updates its amount field based on the account's underlying `lamports`.
/// This is useful if a non-wrapped SOL account uses
/// `system_instruction::transfer` to move lamports to a wrapped token
/// account, and needs to have its token `amount` field updated.
///
/// Accounts expected by this instruction:
///
///   * Using runtime Rent sysvar
///   0. `[writable]`  The native token account to sync with its underlying
///      lamports.
///
///   * Using Rent sysvar account
///   0. `[writable]`  The native token account to sync with its underlying
///      lamports.
///   1. `[]` Rent sysvar.
pub struct SyncNative<'account, Program: TokenProgram> {
    /// Native Token Account
    pub native_token: &'account AccountView,

    pub rent_sysvar: Option<&'account AccountView>,

    _program: PhantomData<Program>,
}

impl<'account, Program: TokenProgram> SyncNative<'account, Program> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = DISCRIMINATOR;

    /// Maximum number of accounts expected by this instruction.
    pub const MAX_ACCOUNTS_LEN: usize = MAX_ACCOUNTS_LEN;

    /// Instruction data length.
    pub const DATA_LEN: usize = DATA_LEN;

    #[inline(always)]
    pub fn new(
        native_token: &'account AccountView,
        rent_sysvar: Option<&'account AccountView>,
    ) -> Self {
        Self {
            native_token,
            rent_sysvar,
            _program: PhantomData,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_with_program(&Program::ID)
    }

    #[inline(always)]
    pub fn invoke_with_program(&self, program: &Address) -> ProgramResult {
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; MAX_ACCOUNTS_LEN];
        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts)?;

        let mut accounts = [UNINIT_CPI_ACCOUNT; MAX_ACCOUNTS_LEN];
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

impl<Program: TokenProgram> CpiWriter for SyncNative<'_, Program> {
    #[inline(always)]
    fn write_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_accounts(self.native_token, self.rent_sysvar, accounts)
    }

    #[inline(always)]
    fn write_instruction_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_instruction_accounts(self.native_token, self.rent_sysvar, accounts)
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        write_instruction_data(data)
    }
}

impl<Program: TokenProgram> super::batch::IntoBatch<Program> for SyncNative<'_, Program> {
    #[inline(always)]
    fn into_batch<'account, 'state>(
        self,
        batch: &mut super::batch::Batch<'account, 'state, Program>,
    ) -> ProgramResult
    where
        Self: 'account + 'state,
    {
        batch.push(
            |accounts| write_accounts(self.native_token, self.rent_sysvar, accounts),
            |accounts| write_instruction_accounts(self.native_token, self.rent_sysvar, accounts),
            write_instruction_data,
        )
    }
}

#[inline(always)]
fn write_accounts<'account, 'out>(
    native_token: &'account AccountView,
    rent_sysvar: Option<&'account AccountView>,
    accounts: &mut [MaybeUninit<CpiAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < MAX_ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    if native_token.is_borrowed() {
        return Err(account_borrow_failed_error());
    }

    CpiAccount::init_from_account_view(native_token, &mut accounts[0]);

    if let Some(rent_sysvar) = rent_sysvar {
        CpiAccount::init_from_account_view(rent_sysvar, &mut accounts[1]);
        Ok(MAX_ACCOUNTS_LEN)
    } else {
        Ok(1)
    }
}

#[inline(always)]
fn write_instruction_accounts<'account, 'out>(
    native_token: &'account AccountView,
    rent_sysvar: Option<&'account AccountView>,
    accounts: &mut [MaybeUninit<InstructionAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < MAX_ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    accounts[0].write(InstructionAccount::writable(native_token.address()));

    if let Some(rent_sysvar) = rent_sysvar {
        accounts[1].write(InstructionAccount::readonly(rent_sysvar.address()));
        Ok(MAX_ACCOUNTS_LEN)
    } else {
        Ok(1)
    }
}

#[inline(always)]
fn write_instruction_data(data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
    if data.len() < DATA_LEN {
        return Err(invalid_argument_error());
    }

    data[0].write(DISCRIMINATOR);

    Ok(DATA_LEN)
}
