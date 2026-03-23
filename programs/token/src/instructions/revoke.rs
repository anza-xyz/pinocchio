use {
    crate::instructions::{
        cpi_account, invalid_argument_error, writable_cpi_account, CpiWriter, MAX_MULTISIG_SIGNERS,
    },
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_signed_unchecked, CpiAccount, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// The number of accounts is 2 for the source and authority, plus the number
/// of multisig signer accounts.
const MAX_ACCOUNTS_LEN: usize = 2 + MAX_MULTISIG_SIGNERS;

const DATA_LEN: usize = 1;

/// Revokes the delegate's authority.
///
/// ### Accounts:
///   * Single owner
///   0. `[WRITE]` The source account.
///   1. `[SIGNER]` The source account owner.
///
///   * Multisignature owner
///   0. `[WRITE]` The source account.
///   1. `[]` The source account's multisignature owner.
///   2. `..2+M` `[SIGNER]` M signer accounts
pub struct Revoke<'account, 'multisig, MultisigSigner: AsRef<AccountView>> {
    /// The source account.
    pub source: &'account AccountView,

    ///  The source account owner.
    pub authority: &'account AccountView,

    /// Multisignature signers.
    pub multisig_signers: &'multisig [MultisigSigner],
}

impl<'account> Revoke<'account, '_, &'account AccountView> {
    /// Creates a new `Revoke` instruction with a single owner authority.
    #[inline(always)]
    pub fn new(source: &'account AccountView, authority: &'account AccountView) -> Self {
        Self::with_multisig_signers(source, authority, &[])
    }
}

impl<'account, 'multisig, MultisigSigner: AsRef<AccountView>>
    Revoke<'account, 'multisig, MultisigSigner>
{
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = 5;

    /// Creates a new `Revoke` instruction with a
    /// multisignature owner authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig_signers(
        source: &'account AccountView,
        authority: &'account AccountView,
        multisig_signers: &'multisig [MultisigSigner],
    ) -> Self {
        Self {
            source,
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
            [const { MaybeUninit::<InstructionAccount>::uninit() }; MAX_ACCOUNTS_LEN];
        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts)?;

        let mut accounts = [const { MaybeUninit::<CpiAccount>::uninit() }; MAX_ACCOUNTS_LEN];
        let written_accounts = self.write_accounts(&mut accounts)?;

        unsafe {
            invoke_signed_unchecked(
                &InstructionView {
                    program_id: &crate::ID,
                    accounts: from_raw_parts(
                        instruction_accounts.as_ptr() as _,
                        written_instruction_accounts,
                    ),
                    data: &[5],
                },
                from_raw_parts(accounts.as_ptr() as _, written_accounts),
                signers,
            );
        }

        Ok(())
    }
}

impl<MultisigSigner: AsRef<AccountView>> CpiWriter for Revoke<'_, '_, MultisigSigner> {
    #[inline(always)]
    fn write_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        'source: 'cpi,
    {
        let expected_accounts = 2 + self.multisig_signers.len();

        if expected_accounts > accounts.len() {
            return Err(invalid_argument_error());
        }

        accounts[0].write(writable_cpi_account(self.source)?);
        accounts[1].write(cpi_account(self.authority)?);

        for (account, signer) in accounts[2..expected_accounts]
            .iter_mut()
            .zip(self.multisig_signers.iter())
        {
            account.write(cpi_account(signer.as_ref())?);
        }

        Ok(expected_accounts)
    }

    #[inline(always)]
    fn write_instruction_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        'source: 'cpi,
    {
        let expected_accounts = 2 + self.multisig_signers.len();

        if expected_accounts > accounts.len() {
            return Err(invalid_argument_error());
        }

        accounts[0].write(InstructionAccount::writable(self.source.address()));
        accounts[1].write(InstructionAccount::new(
            self.authority.address(),
            false,
            self.multisig_signers.is_empty(),
        ));

        for (account, signer) in accounts[2..expected_accounts]
            .iter_mut()
            .zip(self.multisig_signers.iter())
        {
            account.write(InstructionAccount::readonly_signer(
                signer.as_ref().address(),
            ));
        }

        Ok(expected_accounts)
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        if data.len() < DATA_LEN {
            return Err(invalid_argument_error());
        }

        data[0].write(Self::DISCRIMINATOR);

        Ok(DATA_LEN)
    }
}
