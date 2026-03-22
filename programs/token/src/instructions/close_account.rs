use {
    crate::instructions::{Batchable, MAX_MULTISIG_SIGNERS},
    core::{mem::MaybeUninit, ptr, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_signed_unchecked, CpiAccount, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

const CLOSE_ACCOUNT_INSTRUCTION_DATA_LEN: usize = 1;

/// Close an account by transferring all its SOL to the destination account.
///
/// ### Accounts:
///   * Single owner
///   0. `[WRITE]` The account to close.
///   1. `[WRITE]` The destination account.
///   2. `[SIGNER]` The account's owner.
///
///   * Multisignature owner
///   0. `[WRITE]` The account to close.
///   1. `[WRITE]` The destination account.
///   2. `[]` The account's multisignature owner.
///   3. `..3+M` `[SIGNER]` M signer accounts
pub struct CloseAccount<'account, 'multisig, MultisigSigner: AsRef<AccountView>> {
    /// Token Account.
    pub account: &'account AccountView,
    /// Destination Account
    pub destination: &'account AccountView,
    /// Owner Account
    pub authority: &'account AccountView,
    /// Multisignature signers.
    pub multisig_signers: &'multisig [MultisigSigner],
}

impl<'account> CloseAccount<'account, '_, &'account AccountView> {
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
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = 9;

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

        // Instruction accounts.

        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; 3 + MAX_MULTISIG_SIGNERS];

        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts);

        // Accounts.

        let mut accounts =
            [const { MaybeUninit::<CpiAccount>::uninit() }; 3 + MAX_MULTISIG_SIGNERS];

        let written_accounts = self.write_accounts(&mut accounts);

        let mut instruction_data =
            [MaybeUninit::<u8>::uninit(); CLOSE_ACCOUNT_INSTRUCTION_DATA_LEN];
        let written_instruction_data = self.write_instruction_data(&mut instruction_data);

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

impl<MultisigSigner: AsRef<AccountView>> super::sealed::Sealed
    for CloseAccount<'_, '_, MultisigSigner>
{
}

impl<MultisigSigner: AsRef<AccountView>> Batchable for CloseAccount<'_, '_, MultisigSigner> {
    #[inline(always)]
    fn write_accounts(&self, accounts: &mut [MaybeUninit<CpiAccount>]) -> usize {
        let expected_accounts = 3 + self.multisig_signers.len();

        accounts[0].write(CpiAccount::from(self.account));
        accounts[1].write(CpiAccount::from(self.destination));
        accounts[2].write(CpiAccount::from(self.authority));

        for (account, signer) in accounts[3..expected_accounts]
            .iter_mut()
            .zip(self.multisig_signers.iter())
        {
            account.write(CpiAccount::from(signer.as_ref()));
        }

        expected_accounts
    }

    #[inline(always)]
    fn write_instruction_accounts(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount>],
    ) -> usize {
        let expected_accounts = 3 + self.multisig_signers.len();

        // SAFETY: The written address references are borrowed from `self`, and
        // callers must not use the output buffer after `self` expires.
        unsafe {
            ptr::write(
                accounts[0].as_mut_ptr(),
                InstructionAccount::writable(&*(self.account.address() as *const _)),
            );
            ptr::write(
                accounts[1].as_mut_ptr(),
                InstructionAccount::writable(&*(self.destination.address() as *const _)),
            );
            ptr::write(
                accounts[2].as_mut_ptr(),
                InstructionAccount::new(
                    &*(self.authority.address() as *const _),
                    false,
                    self.multisig_signers.is_empty(),
                ),
            );
        }

        for (account, signer) in accounts[3..expected_accounts]
            .iter_mut()
            .zip(self.multisig_signers.iter())
        {
            // SAFETY: Same lifetime requirement as above.
            unsafe {
                ptr::write(
                    account.as_mut_ptr(),
                    InstructionAccount::readonly_signer(&*(signer.as_ref().address() as *const _)),
                );
            }
        }

        expected_accounts
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> usize {
        data[0].write(Self::DISCRIMINATOR);
        CLOSE_ACCOUNT_INSTRUCTION_DATA_LEN
    }
}
