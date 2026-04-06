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

/// This instruction is to be used to rescue SOL sent to any `TokenProgram`
/// owned account by sending them to any other account, leaving behind only
/// lamports for rent exemption.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The source account.
///   1. `[writable]` The destination account.
///   2. `[signer]` The source account's owner/delegate.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The source account.
///   1. `[writable]` The destination account.
///   2. `[]` The source account's multisignature owner/delegate.
///   3. `..+M` `[signer]` M signer accounts.
pub struct WithdrawExcessLamports<'account, 'multisig, MultisigSigner: AsRef<AccountView>> {
    /// Source Account owned by the token program.
    pub source: &'account AccountView,

    /// Destination account.
    pub destination: &'account AccountView,

    /// Authority.
    pub authority: &'account AccountView,

    /// Multisignature signers.
    pub multisig_signers: &'multisig [MultisigSigner],
}

impl<'account> WithdrawExcessLamports<'account, '_, &'account AccountView> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = 38;

    /// Maximum number of accounts expected by this instruction.
    ///
    /// The required number of accounts will depend whether the
    /// source account has a single owner or a multisignature
    /// owner.
    pub const MAX_ACCOUNTS_LEN: usize = 3 + MAX_MULTISIG_SIGNERS;

    /// Instruction data length:
    ///   - discriminator (1 byte)
    pub const DATA_LEN: usize = 1;

    /// Creates a new `WithdrawExcessLamports` instruction with a single owner
    /// authority.
    #[inline(always)]
    pub fn new(
        source: &'account AccountView,
        destination: &'account AccountView,
        authority: &'account AccountView,
    ) -> Self {
        Self::with_multisig_signers(source, destination, authority, &[])
    }
}

impl<'account, 'multisig, MultisigSigner: AsRef<AccountView>>
    WithdrawExcessLamports<'account, 'multisig, MultisigSigner>
{
    /// Creates a new `WithdrawExcessLamports` instruction with a
    /// multisignature owner authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig_signers(
        source: &'account AccountView,
        destination: &'account AccountView,
        authority: &'account AccountView,
        multisig_signers: &'multisig [MultisigSigner],
    ) -> Self {
        Self {
            source,
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

        let mut instruction_accounts =
            [UNINIT_INSTRUCTION_ACCOUNT; WithdrawExcessLamports::MAX_ACCOUNTS_LEN];
        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts)?;

        let mut accounts = [UNINIT_CPI_ACCOUNT; WithdrawExcessLamports::MAX_ACCOUNTS_LEN];
        let written_accounts = self.write_accounts(&mut accounts)?;

        let mut instruction_data = [UNINIT_BYTE; WithdrawExcessLamports::DATA_LEN];
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
                from_raw_parts(accounts.as_ptr() as _, written_accounts),
                signers,
            );
        }

        Ok(())
    }
}

impl<MultisigSigner: AsRef<AccountView>> CpiWriter
    for WithdrawExcessLamports<'_, '_, MultisigSigner>
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
            self.source,
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
            self.source,
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

impl<MultisigSigner: AsRef<AccountView>> super::IntoBatch
    for WithdrawExcessLamports<'_, '_, MultisigSigner>
{
    #[inline(always)]
    fn into_batch<'account, 'state>(
        self,
        batch: &mut super::Batch<'account, 'state>,
    ) -> ProgramResult
    where
        Self: 'account + 'state,
    {
        batch.push(
            |accounts| {
                write_accounts(
                    self.source,
                    self.destination,
                    self.authority,
                    self.multisig_signers,
                    accounts,
                )
            },
            |accounts| {
                write_instruction_accounts(
                    self.source,
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
    source: &'account AccountView,
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

    if source.is_borrowed() | destination.is_borrowed() {
        return Err(account_borrow_failed_error());
    }

    CpiAccount::init_from_account_view(source, &mut accounts[0]);

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
    source: &'account AccountView,
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

    accounts[0].write(InstructionAccount::writable(source.address()));

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
    if data.len() < WithdrawExcessLamports::DATA_LEN {
        return Err(invalid_argument_error());
    }

    data[0].write(WithdrawExcessLamports::DISCRIMINATOR);

    Ok(WithdrawExcessLamports::DATA_LEN)
}
