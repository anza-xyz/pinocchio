use {
    crate::instructions::{extensions::ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
    core::{mem::MaybeUninit, slice},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Lock certain token operations from taking place within CPI for this
/// Account, namely:
/// * `Transfer` and `Burn` must go through a delegate.
/// * `CloseAccount` can only return lamports to owner.
/// * `SetAuthority` can only be used to remove an existing close authority.
/// * `Approve` is disallowed entirely.
///
/// In addition, CPI Guard cannot be enabled or disabled via CPI.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The account to update.
///   1. `[signer]` The account's owner.
///
///   * Multisignature authority
///   0. `[writable]` The account to update.
///   1. `[]` The account's multisignature owner.
///   2. `..2+M` `[signer]` M signer accounts.
pub struct Enable<'a, 'b, 'c> {
    /// The account to update.
    pub account: &'a AccountView,

    /// The account's owner.
    pub authority: &'a AccountView,

    /// The signer accounts if the authority is a multisig.
    pub signers: &'c [&'a AccountView],

    /// The token program.
    pub token_program: &'b Address,
}

impl<'a, 'b, 'c> Enable<'a, 'b, 'c> {
    pub const DISCRIMINATOR: u8 = 0;

    /// Creates a new `Enable` instruction with a single owner/delegate
    /// authority.
    #[inline(always)]
    pub fn new(
        token_program: &'b Address,
        account: &'a AccountView,
        authority: &'a AccountView,
    ) -> Self {
        Self::with_signers(token_program, account, authority, &[])
    }

    /// Creates a new `Enable` instruction with a multisignature owner/delegate
    /// authority and signer accounts.
    #[inline(always)]
    pub fn with_signers(
        token_program: &'b Address,
        account: &'a AccountView,
        authority: &'a AccountView,
        signers: &'c [&'a AccountView],
    ) -> Self {
        Self {
            account,
            authority,
            signers,
            token_program,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let expected_accounts = 2 + self.signers.len();

        // Instruction accounts.

        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.account.address()));

            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::new(
                    self.authority.address(),
                    false,
                    self.signers.is_empty(),
                ));

            for (account, signer) in instruction_accounts
                .get_unchecked_mut(2..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // Accounts.

        const UNINIT_ACCOUNT_VIEWS: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut accounts = [UNINIT_ACCOUNT_VIEWS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            accounts.get_unchecked_mut(0).write(self.account);

            accounts.get_unchecked_mut(1).write(self.authority);

            for (account, signer) in accounts
                .get_unchecked_mut(2..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(signer);
            }
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &InstructionView {
                program_id: self.token_program,
                // SAFETY: instruction accounts has `expected_accounts` initialized.
                accounts: unsafe {
                    slice::from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                data: &[ExtensionDiscriminator::CpiGuard as u8, Self::DISCRIMINATOR],
            },
            // SAFETY: accounts has `expected_accounts` initialized.
            unsafe {
                slice::from_raw_parts(accounts.as_ptr() as *const &AccountView, expected_accounts)
            },
            signers,
        )
    }
}
