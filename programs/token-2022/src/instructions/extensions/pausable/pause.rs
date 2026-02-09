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

/// Pause minting, burning, and transferring for the mint.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint to update.
///   1. `[signer]` The mint's pause authority.
///
///   * Multisignature authority
///   0. `[writable]` The mint to update.
///   1. `[]` The mint's multisignature pause authority.
///   2. `..2+M` `[signer]` M signer accounts.
pub struct Pause<'a, 'b, 'c> {
    /// The mint to update.
    pub mint: &'a AccountView,

    /// The mint's pause authority.
    pub authority: &'a AccountView,

    /// The signer accounts if the authority is a multisig.
    pub signers: &'c [&'a AccountView],

    /// The token program.
    pub token_program: &'b Address,
}

impl<'a, 'b, 'c> Pause<'a, 'b, 'c> {
    pub const DISCRIMINATOR: u8 = 1;

    /// Creates a new `Pause` instruction with a single owner/delegate
    /// authority.
    #[inline(always)]
    pub fn new(
        token_program: &'b Address,
        mint: &'a AccountView,
        authority: &'a AccountView,
    ) -> Self {
        Self::with_signers(token_program, mint, authority, &[])
    }

    /// Creates a new `Pause` instruction with a multisignature owner/delegate
    /// authority and signer accounts.
    #[inline(always)]
    pub fn with_signers(
        token_program: &'b Address,
        mint: &'a AccountView,
        authority: &'a AccountView,
        signers: &'c [&'a AccountView],
    ) -> Self {
        Self {
            mint,
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
                .write(InstructionAccount::writable(self.mint.address()));

            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::new(
                    self.authority.address(),
                    false,
                    self.signers.is_empty(),
                ));

            for (account, signer) in instruction_accounts[2..]
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
            accounts.get_unchecked_mut(0).write(self.mint);

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
                data: &[ExtensionDiscriminator::Pausable as u8, Self::DISCRIMINATOR],
            },
            // SAFETY: accounts has `expected_accounts` initialized.
            unsafe { slice::from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
