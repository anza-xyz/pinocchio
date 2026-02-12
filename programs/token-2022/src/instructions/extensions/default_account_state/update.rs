use {
    crate::{
        instructions::{extensions::ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
        state::AccountState,
    },
    core::{mem::MaybeUninit, slice},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Update the default state for new Accounts. Only supported for mints that
/// include the `DefaultAccountState` extension.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The mint.
///   1. `[signer]` The mint freeze authority.
///
///   * Multisignature authority
///   0. `[writable]` The mint.
///   1. `[]` The mint's multisignature freeze authority.
///   2. `..2+M` `[signer]` M signer accounts.
pub struct Update<'a, 'b, 'c> {
    /// The mint.
    pub mint: &'a AccountView,

    /// The mint freeze authority.
    pub freeze_authority: &'a AccountView,

    /// The signer accounts if the authority is a multisig.
    pub multisig_signers: &'c [&'a AccountView],

    /// The new account state in which new token accounts should be
    /// initialized.
    pub state: AccountState,

    /// The token program.
    pub token_program: &'b Address,
}

impl<'a, 'b, 'c> Update<'a, 'b, 'c> {
    pub const DISCRIMINATOR: u8 = 1;

    /// Creates a new `Update` instruction with a single owner/delegate
    /// authority.
    #[inline(always)]
    pub fn new(
        token_program: &'b Address,
        mint: &'a AccountView,
        freeze_authority: &'a AccountView,
        state: AccountState,
    ) -> Self {
        Self::with_multisig_signers(token_program, mint, freeze_authority, state, &[])
    }

    /// Creates a new `Update` instruction with a multisignature owner/delegate
    /// authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig_signers(
        token_program: &'b Address,
        mint: &'a AccountView,
        freeze_authority: &'a AccountView,
        state: AccountState,
        multisig_signers: &'c [&'a AccountView],
    ) -> Self {
        Self {
            mint,
            freeze_authority,
            multisig_signers,
            state,
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

        instruction_accounts[0].write(InstructionAccount::writable(self.mint.address()));

        instruction_accounts[1].write(InstructionAccount::new(
            self.freeze_authority.address(),
            false,
            self.multisig_signers.is_empty(),
        ));

        for (account, signer) in instruction_accounts[2..]
            .iter_mut()
            .zip(self.multisig_signers.iter())
        {
            account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        // Accounts.

        let mut accounts =
            [const { MaybeUninit::<&AccountView>::uninit() }; 2 + MAX_MULTISIG_SIGNERS];

        accounts[0].write(self.mint);

        accounts[1].write(self.freeze_authority);

        for (account, signer) in accounts[2..].iter_mut().zip(self.multisig_signers.iter()) {
            account.write(signer);
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &InstructionView {
                program_id: self.token_program,
                // SAFETY: instruction accounts has `expected_accounts` initialized.
                accounts: unsafe {
                    slice::from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                data: &[
                    ExtensionDiscriminator::DefaultAccountState as u8,
                    Self::DISCRIMINATOR,
                    self.state as u8,
                ],
            },
            // SAFETY: accounts has `expected_accounts` initialized.
            unsafe { slice::from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
