use {
    crate::{instructions::extensions::ExtensionDiscriminator, instructions::MAX_MULTISIG_SIGNERS},
    core::{mem::MaybeUninit, slice},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Disable the CPI Guard extension on a token account.
///
/// Expected accounts:
///
/// **Single authority**
/// 0. `[writable]` The token account to enable cpi-guard.
/// 1. `[signer]` The owner of the token account.
///
/// **Multisignature authority**
/// 0. `[writable]` The token account to enable cpi-guard.
/// 1. `[readonly]` The multisig account that owns the token account.
/// 2. `[signer]` M signer accounts (as required by the multisig).
pub struct DisableCpiGuard<'a, 'b, 'c> {
    /// The token account to enable with the CpiGuard extension.
    pub token_account: &'a AccountView,
    /// The owner of the token account (single or multisig).
    pub authority: &'a AccountView,
    /// Signer accounts if the authority is a multisig.
    pub signers: &'c [&'a AccountView],
    /// Token program (Token-2022).
    pub token_program: &'b Address,
}

impl DisableCpiGuard<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 1;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            token_account,
            authority,
            signers: multisig_accounts,
            token_program,
            ..
        } = self;

        if multisig_accounts.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        // creates an array of uninitialized InstructionAccount with 2 + MAX_MULTISIG_SIGNERS
        // i.e. [token_account + authority](2) + signers(max_multisig_signers)
        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY:
        // - `accounts` is sized to 2 + MAX_MULTISIG_SIGNERS
        // - Index 0 is always present (TokenAccount)
        // - Index 1 is always present (Authority)
        unsafe {
            accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(token_account.address()));

            accounts.get_unchecked_mut(1).write(InstructionAccount::new(
                authority.address(),
                false,
                multisig_accounts.is_empty(),
            ));
        }

        // add the multisig if they exist for each signer account
        // creates a tuple of (account, signer) for each multisig i.e from index 2
        for (account, signer) in accounts[2..].iter_mut().zip(multisig_accounts.iter()) {
            account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        // build instruction data for CpiGuard
        let data = &[ExtensionDiscriminator::CpiGuard as u8, Self::DISCRIMINATOR];

        let num_accounts = 2 + multisig_accounts.len();

        // build instruction for CpiGuard
        let instruction = InstructionView {
            program_id: token_program,
            data,
            accounts: unsafe {
                // create a slice &[] by providing the pointer to that data and the length of the data
                slice::from_raw_parts(accounts.as_ptr() as _, num_accounts)
            },
        };

        // Account view array
        const UNINIT_ACCOUNT_VIEWS: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut account_views = [UNINIT_ACCOUNT_VIEWS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY:
        // - `account_views` is sized to 2 + MAX_MULTISIG_SIGNERS
        // - Index 0 is always present
        // - Index 1 is always present
        unsafe {
            account_views.get_unchecked_mut(0).write(token_account);
            account_views.get_unchecked_mut(1).write(authority);
        }

        // Fill all signer accounts
        for (account_view, signer) in account_views[2..].iter_mut().zip(multisig_accounts.iter()) {
            account_view.write(signer);
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe {
                slice::from_raw_parts(account_views.as_ptr() as *const &AccountView, num_accounts)
            },
            signers,
        )
    }
}
