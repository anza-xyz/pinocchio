use {
    crate::instructions::MAX_MULTISIG_SIGNERS,
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

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
pub struct CloseAccount<'a, 'b> {
    /// Token Account.
    pub account: &'a AccountView,
    /// Destination Account
    pub destination: &'a AccountView,
    /// Owner Account
    pub authority: &'a AccountView,
    /// Multisignature signers.
    pub multisig_signers: &'b [&'a AccountView],
}

impl<'a, 'b> CloseAccount<'a, 'b> {
    /// Creates a new `CloseAccount` instruction with a single owner authority.
    #[inline(always)]
    pub fn new(
        account: &'a AccountView,
        destination: &'a AccountView,
        authority: &'a AccountView,
    ) -> Self {
        Self::with_multisig_signers(account, destination, authority, &[])
    }

    /// Creates a new `CloseAccount` instruction with a
    /// multisignature owner authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig_signers(
        account: &'a AccountView,
        destination: &'a AccountView,
        authority: &'a AccountView,
        multisig_signers: &'b [&'a AccountView],
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

        let expected_accounts = 3 + self.multisig_signers.len();

        // Instruction accounts.

        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; 3 + MAX_MULTISIG_SIGNERS];

        instruction_accounts[0].write(InstructionAccount::writable(self.account.address()));

        instruction_accounts[1].write(InstructionAccount::writable(self.destination.address()));

        instruction_accounts[2].write(InstructionAccount::new(
            self.authority.address(),
            false,
            self.multisig_signers.is_empty(),
        ));

        for (account, signer) in instruction_accounts[3..]
            .iter_mut()
            .zip(self.multisig_signers.iter())
        {
            account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        // Accounts.

        let mut accounts =
            [const { MaybeUninit::<&AccountView>::uninit() }; 3 + MAX_MULTISIG_SIGNERS];

        accounts[0].write(self.account);

        accounts[1].write(self.destination);

        accounts[2].write(self.authority);

        for (account, signer) in accounts[3..].iter_mut().zip(self.multisig_signers.iter()) {
            account.write(signer);
        }

        invoke_signed_with_bounds::<{ 3 + MAX_MULTISIG_SIGNERS }>(
            &InstructionView {
                program_id: &crate::ID,
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                data: &[9],
            },
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
