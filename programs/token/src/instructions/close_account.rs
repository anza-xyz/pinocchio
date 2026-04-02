use {
    crate::{
        instructions::{
            account_borrow_failed_error, invalid_argument_error, CpiWriter, MAX_MULTISIG_SIGNERS,
        },
        UNINIT_BYTE, UNINIT_CPI_ACCOUNT, UNINIT_INSTRUCTION_ACCOUNT,
    },
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_signed_unchecked, CpiAccount, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Maximum number of accounts expected by this instruction.
///
/// The required number of accounts will depend whether the
/// source account has a single owner or a multisignature
/// owner.
const MAX_ACCOUNTS_LEN: usize = 3 + MAX_MULTISIG_SIGNERS;

/// Instruction data length:
///   - discriminator (1 byte)
const DATA_LEN: usize = 1;

/// Close an account by transferring all its SOL to the destination account.
/// Non-native accounts may only be closed if its token amount is zero.
///
/// Accounts expected by this instruction:
///
///   * Single owner
///   0. `[writable]` The account to close.
///   1. `[writable]` The destination account.
///   2. `[signer]` The account's owner.
///
///   * Multisignature owner
///   0. `[writable]` The account to close.
///   1. `[writable]` The destination account.
///   2. `[]` The account's multisignature owner.
///   3. `..+M` `[signer]` M signer accounts.
pub struct CloseAccount<'account, 'multisig, MultisigSigner: AsRef<AccountView>> {
    /// The account to close.
    pub account: &'account AccountView,

    /// The destination account.
    pub destination: &'account AccountView,

    /// The account's owner.
    pub authority: &'account AccountView,

    /// Multisignature signers.
    pub multisig_signers: &'multisig [MultisigSigner],
}

impl<'account> CloseAccount<'account, '_, &'account AccountView> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = 9;

    /// Creates a new `CloseAccount` instruction with a single owner authority.
    #[inline(always)]
    pub fn new(
        account: &'account AccountView,
        destination: &'account AccountView,
        authority: &'account AccountView,
    ) -> Self {
        Self::with_multisig_signers(account, destination, authority, &[])
    }
}

impl<'account, 'multisig, MultisigSigner: AsRef<AccountView>>
    CloseAccount<'account, 'multisig, MultisigSigner>
{
    /// Creates a new `CloseAccount` instruction with a
    /// multisignature owner authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig_signers(
        account: &'account AccountView,
        destination: &'account AccountView,
        authority: &'account AccountView,
        multisig_signers: &'multisig [MultisigSigner],
    ) -> Self {
        Self {
            account,
            destination,
            authority,
            multisig_signers,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            Err(ProgramError::InvalidArgument)?;
        }

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; MAX_ACCOUNTS_LEN];
        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts)?;

        let mut accounts = [UNINIT_CPI_ACCOUNT; MAX_ACCOUNTS_LEN];
        let written_accounts = self.write_accounts(&mut accounts)?;

        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];
        let written_instruction_data = self.write_instruction_data(&mut instruction_data)?;

        unsafe {
            invoke_signed_unchecked(
                &InstructionView {
                    program_id: &crate::ID,
                    accounts: from_raw_parts(
                        instruction_accounts.as_ptr() as _,
                        written_instruction_accounts,
                    ),
                    data: from_raw_parts(instruction_data.as_ptr() as _, written_instruction_data),
                },
                from_raw_parts(accounts.as_ptr() as *const CpiAccount, written_accounts),
                signers,
            );
        }

        Ok(())
    }
}

impl<MultisigSigner: AsRef<AccountView>> CpiWriter for CloseAccount<'_, '_, MultisigSigner> {
    #[inline(always)]
    fn write_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_accounts(
            self.account,
            self.destination,
            self.authority,
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
            self.account,
            self.destination,
            self.authority,
            self.multisig_signers,
            accounts,
        )
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        write_instruction_data(data)
    }
}

#[cfg(feature = "batch")]
impl<MultisigSigner: AsRef<AccountView>> super::IntoBatch for CloseAccount<'_, '_, MultisigSigner> {
    #[inline(always)]
    fn into_batch<'batch>(self, batch: &mut super::Batch<'batch>) -> ProgramResult
    where
        Self: 'batch,
    {
        batch.push(
            |accounts| {
                write_accounts(
                    self.account,
                    self.destination,
                    self.authority,
                    self.multisig_signers,
                    accounts,
                )
            },
            |accounts| {
                write_instruction_accounts(
                    self.account,
                    self.destination,
                    self.authority,
                    self.multisig_signers,
                    accounts,
                )
            },
            write_instruction_data,
        )
    }
}

#[inline(always)]
fn write_accounts<'account, 'multisig, 'out, MultisigSigner: AsRef<AccountView>>(
    account: &'account AccountView,
    destination: &'account AccountView,
    authority: &'account AccountView,
    multisig_signers: &'multisig [MultisigSigner],
    accounts: &mut [MaybeUninit<CpiAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
    'multisig: 'out,
{
    let expected_accounts = 3 + multisig_signers.len();

    if expected_accounts > accounts.len() {
        return Err(invalid_argument_error());
    }

    if account.is_borrowed() | destination.is_borrowed() {
        return Err(account_borrow_failed_error());
    }

    CpiAccount::init_from_account_view(account, &mut accounts[0]);

    CpiAccount::init_from_account_view(destination, &mut accounts[1]);

    CpiAccount::init_from_account_view(authority, &mut accounts[2]);

    for (account, signer) in accounts[3..expected_accounts]
        .iter_mut()
        .zip(multisig_signers.iter())
    {
        CpiAccount::init_from_account_view(signer.as_ref(), account);
    }

    Ok(expected_accounts)
}

#[inline(always)]
fn write_instruction_accounts<'account, 'multisig, 'out, MultisigSigner: AsRef<AccountView>>(
    account: &'account AccountView,
    destination: &'account AccountView,
    authority: &'account AccountView,
    multisig_signers: &'multisig [MultisigSigner],
    accounts: &mut [MaybeUninit<InstructionAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
    'multisig: 'out,
{
    let expected_accounts = 3 + multisig_signers.len();

    if expected_accounts > accounts.len() {
        return Err(invalid_argument_error());
    }

    accounts[0].write(InstructionAccount::writable(account.address()));

    accounts[1].write(InstructionAccount::writable(destination.address()));

    accounts[2].write(InstructionAccount::new(
        authority.address(),
        false,
        multisig_signers.is_empty(),
    ));

    for (account, signer) in accounts[3..expected_accounts]
        .iter_mut()
        .zip(multisig_signers.iter())
    {
        account.write(InstructionAccount::readonly_signer(
            signer.as_ref().address(),
        ));
    }

    Ok(expected_accounts)
}

#[inline(always)]
fn write_instruction_data(data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
    if data.len() < DATA_LEN {
        return Err(invalid_argument_error());
    }

    data[0].write(CloseAccount::DISCRIMINATOR);

    Ok(DATA_LEN)
}
