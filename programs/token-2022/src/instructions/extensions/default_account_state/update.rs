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

/// Update the Default-Account-State extension on a mint.
///
/// Expected accounts:
///
/// **Single authority**
/// 0. `[writable]` The mint account to update.
/// 1. `[signer]` The freeze authority of the mint.
///
/// **Multisignature authority**
/// 0. `[writable]` The mint account to update.
/// 1. `[readonly]` The multisig freeze authority.
/// 2. `[signer]` M signer accounts (as required by the multisig).
pub struct Update<'a, 'b, 'c> {
    /// The mint account to update.
    pub mint_account: &'a AccountView,
    /// The freeze authority of the mint (single or multisig).
    pub freeze_authority: &'a AccountView,
    /// Signer accounts if the authority is a multisig.
    pub signers: &'c [&'a AccountView],
    /// The default account state in which new token accounts should be initialized.
    pub state: u8,
    /// Token program (Token-2022).
    pub token_program: &'b Address,
}

impl Update<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 1;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            mint_account,
            freeze_authority,
            signers: multisig_accounts,
            token_program,
            ..
        } = self;

        if multisig_accounts.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY:
        // - `account_metas` is sized to 2 + MAX_MULTISIG_SIGNERS
        // Index 0 and 1 are always present
        unsafe {
            accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(mint_account.address()));

            accounts.get_unchecked_mut(1).write(InstructionAccount::new(
                freeze_authority.address(),
                false,
                multisig_accounts.is_empty(),
            ));
        }

        for (account, signer) in accounts[2..].iter_mut().zip(multisig_accounts.iter()) {
            account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        let data = &[
            ExtensionDiscriminator::DefaultAccountState as u8,
            Self::DISCRIMINATOR,
            self.state,
        ];

        let num_accounts = 2 + multisig_accounts.len();

        let instruction = InstructionView {
            program_id: token_program,
            data,
            accounts: unsafe { slice::from_raw_parts(accounts.as_ptr() as _, num_accounts) },
        };

        // Account view array
        const UNINIT_ACCOUNT_VIEWS: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut account_views = [UNINIT_ACCOUNT_VIEWS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY:
        // - `account_views` is sized to 2 + MAX_MULTISIG_SIGNERS
        // Index 0 and 1 are always present
        unsafe {
            account_views.get_unchecked_mut(0).write(mint_account);
            account_views.get_unchecked_mut(1).write(freeze_authority);
        }

        // Fill signer accounts
        for (account_view, signer) in account_views[2..].iter_mut().zip(multisig_accounts.iter()) {
            account_view.write(signer);
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { slice::from_raw_parts(account_views.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}
