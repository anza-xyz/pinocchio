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

/// Maximum number of multisignature signers.
pub const MAX_MULTISIG_SIGNERS: usize = 11;

/// The instruction discriminator.
const DISCRIMINATOR: u8 = 2;

/// Maximum number of accounts expected by this instruction.
///
/// The required number of accounts will depend on the number of signer
/// accounts.
const MAX_ACCOUNTS_LEN: usize = 2 + MAX_MULTISIG_SIGNERS;

/// Instruction data length:
///   - discriminator (1 byte)
///   - number of signers (1 byte)
const DATA_LEN: usize = 2;

/// Initializes a multisignature account with N provided signers.
///
/// Multisignature accounts can used in place of any single owner/delegate
/// accounts in any token instruction that require an owner/delegate to be
/// present.  The variant field represents the number of signers (M)
/// required to validate this multisignature account.
///
/// The [`super::InitializeMultisig`] instruction requires no
/// signers and MUST be included within the same Transaction as the
/// system program's `CreateAccount` instruction that creates the
/// account being initialized. Otherwise another party can acquire
/// ownership of the uninitialized account.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The multisignature account to initialize.
///   1. `[]` Rent sysvar.
///   2. `..+N` `[signer]` The signer accounts, must equal to N where `1 <= N <=
///      11`.
pub struct InitializeMultisig<
    'account,
    'multisig,
    MultisigSigner: AsRef<AccountView>,
    Program: TokenProgram,
> where
    'account: 'multisig,
{
    /// The multisignature account to initialize.
    pub multisig: &'account AccountView,

    /// Rent sysvar.
    pub rent_sysvar: &'account AccountView,

    /// The signer accounts.
    pub multisig_signers: &'multisig [MultisigSigner],

    /// The number of signers (M) required to validate this multisignature
    /// account.
    pub m: u8,

    _program: PhantomData<Program>,
}

impl<'account, 'multisig, MultisigSigner: AsRef<AccountView>, Program: TokenProgram>
    InitializeMultisig<'account, 'multisig, MultisigSigner, Program>
where
    'account: 'multisig,
{
    /// Maximum number of multisignature signers.
    pub const MAX_MULTISIG_SIGNERS: usize = MAX_MULTISIG_SIGNERS;

    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = DISCRIMINATOR;

    /// Maximum number of accounts expected by this instruction.
    pub const MAX_ACCOUNTS_LEN: usize = MAX_ACCOUNTS_LEN;

    /// Instruction data length.
    pub const DATA_LEN: usize = DATA_LEN;

    #[inline(always)]
    pub fn new(
        multisig: &'account AccountView,
        rent_sysvar: &'account AccountView,
        multisig_signers: &'multisig [MultisigSigner],
        m: u8,
    ) -> Self {
        Self {
            multisig,
            rent_sysvar,
            multisig_signers,
            m,
            _program: PhantomData,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_with_program(&Program::ID)
    }

    #[inline(always)]
    pub fn invoke_with_program(&self, program: &Address) -> ProgramResult {
        if self.multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

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

impl<MultisigSigner: AsRef<AccountView>, Program: TokenProgram> CpiWriter
    for InitializeMultisig<'_, '_, MultisigSigner, Program>
{
    #[inline(always)]
    fn write_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_accounts(
            self.multisig,
            self.rent_sysvar,
            self.multisig_signers,
            accounts,
        )
    }

    #[inline(always)]
    fn write_instruction_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_instruction_accounts(
            self.multisig,
            self.rent_sysvar,
            self.multisig_signers,
            accounts,
        )
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        write_instruction_data(self.m, data)
    }
}

impl<MultisigSigner: AsRef<AccountView>, Program: TokenProgram> super::batch::IntoBatch<Program>
    for InitializeMultisig<'_, '_, MultisigSigner, Program>
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
            |accounts| {
                write_accounts(
                    self.multisig,
                    self.rent_sysvar,
                    self.multisig_signers,
                    accounts,
                )
            },
            |accounts| {
                write_instruction_accounts(
                    self.multisig,
                    self.rent_sysvar,
                    self.multisig_signers,
                    accounts,
                )
            },
            |data| write_instruction_data(self.m, data),
        )
    }
}

#[inline(always)]
fn write_accounts<'account, 'multisig, 'out, MultisigSigner: AsRef<AccountView>>(
    multisig: &'account AccountView,
    rent_sysvar: &'account AccountView,
    multisig_signers: &'multisig [MultisigSigner],
    accounts: &mut [MaybeUninit<CpiAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
    'multisig: 'out,
{
    let expected_accounts = 2 + multisig_signers.len();

    if expected_accounts > accounts.len() {
        return Err(invalid_argument_error());
    }

    if multisig.is_borrowed() {
        return Err(account_borrow_failed_error());
    }

    CpiAccount::init_from_account_view(multisig, &mut accounts[0]);

    CpiAccount::init_from_account_view(rent_sysvar, &mut accounts[1]);

    for (account, signer) in accounts[2..expected_accounts]
        .iter_mut()
        .zip(multisig_signers.iter())
    {
        CpiAccount::init_from_account_view(signer.as_ref(), account);
    }

    Ok(expected_accounts)
}

#[inline(always)]
fn write_instruction_accounts<'account, 'multisig, 'out, MultisigSigner: AsRef<AccountView>>(
    multisig: &'account AccountView,
    rent_sysvar: &'account AccountView,
    multisig_signers: &'multisig [MultisigSigner],
    accounts: &mut [MaybeUninit<InstructionAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
    'multisig: 'out,
{
    let expected_accounts = 2 + multisig_signers.len();

    if expected_accounts > accounts.len() {
        return Err(invalid_argument_error());
    }

    accounts[0].write(InstructionAccount::writable(multisig.address()));

    accounts[1].write(InstructionAccount::readonly(rent_sysvar.address()));

    for (account, signer) in accounts[2..expected_accounts]
        .iter_mut()
        .zip(multisig_signers.iter())
    {
        account.write(InstructionAccount::readonly(signer.as_ref().address()));
    }

    Ok(expected_accounts)
}

#[inline(always)]
fn write_instruction_data(m: u8, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
    if data.len() < DATA_LEN {
        return Err(invalid_argument_error());
    }

    data[0].write(DISCRIMINATOR);

    data[1].write(m);

    Ok(DATA_LEN)
}
