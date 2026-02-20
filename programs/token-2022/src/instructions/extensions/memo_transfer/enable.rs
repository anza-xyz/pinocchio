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

/// Require memos for transfers into this Account. Adds the `MemoTransfer`
/// extension to the Account, if it doesn't already exist.
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

    /// Signer accounts if the authority is a multisig.
    pub multisig_signers: &'c [&'a AccountView],

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
        Self::with_multisig_signers(token_program, account, authority, &[])
    }

    /// Creates a new `Enable` instruction with a multisignature owner/delegate
    /// authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig_signers(
        token_program: &'b Address,
        account: &'a AccountView,
        authority: &'a AccountView,
        multisig_signers: &'c [&'a AccountView],
    ) -> Self {
        Self {
            account,
            authority,
            multisig_signers,
            token_program,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let expected_accounts = 2 + self.multisig_signers.len();

        // Instruction accounts.

        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; 2 + MAX_MULTISIG_SIGNERS];

        instruction_accounts[0].write(InstructionAccount::writable(self.account.address()));

        instruction_accounts[1].write(InstructionAccount::new(
            self.authority.address(),
            false,
            self.multisig_signers.is_empty(),
        ));

        for (instruction_account, signer) in instruction_accounts[2..]
            .iter_mut()
            .zip(self.multisig_signers.iter())
        {
            instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        // Accounts.

        let mut accounts = [MaybeUninit::<&AccountView>::uninit(); 2 + MAX_MULTISIG_SIGNERS];

        accounts[0].write(self.account);

        accounts[1].write(self.authority);

        for (account_view, signer) in accounts[2..].iter_mut().zip(self.multisig_signers.iter()) {
            account_view.write(signer);
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &InstructionView {
                program_id: self.token_program,
                // SAFETY: instruction accounts has `expected_accounts` initialized.
                accounts: unsafe {
                    slice::from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                data: &[
                    ExtensionDiscriminator::MemoTransfer as u8,
                    Self::DISCRIMINATOR,
                ],
            },
            // SAFETY: accounts has `expected_accounts` initialized.
            unsafe {
                slice::from_raw_parts(accounts.as_ptr() as *const &AccountView, expected_accounts)
            },
            signers,
        )
    }
}
