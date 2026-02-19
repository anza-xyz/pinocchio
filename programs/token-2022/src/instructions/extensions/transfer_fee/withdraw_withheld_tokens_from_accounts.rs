use {
    crate::instructions::{ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer, MAX_STATIC_CPI_ACCOUNTS},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Transfer all withheld tokens to an account. Signed by the mint's
/// withdraw withheld tokens authority.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[]` The token mint. Must include the `TransferFeeConfig` extension.
///   1. `[writable]` The fee receiver account. Must include the
///      `TransferFeeAmount` extension and be associated with the provided mint.
///   2. `[signer]` The mint's `withdraw_withheld_authority`.
///   3. `..3+N` `[writable]` The source accounts to withdraw from.
///
///   * Multisignature owner/delegate
///   0. `[]` The token mint.
///   1. `[writable]` The destination account.
///   2. `[]` The mint's multisig `withdraw_withheld_authority`.
///   3. `..3+M` `[signer]` M signer accounts.
///   4. `3+M+1..3+M+N` `[writable]` The source accounts to withdraw from.
pub struct WithdrawWithheldTokensFromAccounts<'a, 'b, 'c> {
    /// The token mint.
    pub mint: &'a AccountView,

    /// The fee receiver account.
    pub destination: &'a AccountView,

    /// The mint's `withdraw_withheld_authority` or multisig.
    pub authority: &'a AccountView,

    /// Multisignature signer accounts.
    pub multisig_signers: &'c [&'a AccountView],

    /// Source accounts to withdraw from.
    pub sources: &'c [&'a AccountView],

    /// Token program.
    pub token_program: &'b Address,
}

impl<'a, 'b, 'c> WithdrawWithheldTokensFromAccounts<'a, 'b, 'c> {
    pub const DISCRIMINATOR: u8 = 3;

    /// Creates a new `WithdrawWithheldTokensFromAccounts` instruction
    /// with a single owner/delegate authority.
    #[inline(always)]
    pub fn new(
        token_program: &'b Address,
        mint: &'a AccountView,
        destination: &'a AccountView,
        authority: &'a AccountView,
        sources: &'c [&'a AccountView],
    ) -> Self {
        Self::with_multisig_signers(token_program, mint, destination, authority, sources, &[])
    }

    /// Creates a new `WithdrawWithheldTokensFromAccounts` instruction with a
    /// multisignature owner/delegate authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig_signers(
        token_program: &'b Address,
        mint: &'a AccountView,
        destination: &'a AccountView,
        authority: &'a AccountView,
        sources: &'c [&'a AccountView],
        multisig_signers: &'c [&'a AccountView],
    ) -> Self {
        Self {
            mint,
            destination,
            authority,
            multisig_signers,
            sources,
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

        let expected_accounts = 3 + self.multisig_signers.len() + self.sources.len();

        if expected_accounts > MAX_STATIC_CPI_ACCOUNTS {
            return Err(ProgramError::InvalidArgument);
        }

        // Instruction accounts.

        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; MAX_STATIC_CPI_ACCOUNTS];

        instruction_accounts[0].write(InstructionAccount::writable(self.mint.address()));

        instruction_accounts[1].write(InstructionAccount::writable(self.destination.address()));

        instruction_accounts[2].write(InstructionAccount::new(
            self.authority.address(),
            false,
            self.multisig_signers.is_empty(),
        ));

        for (instruction_account, signer) in instruction_accounts[3..]
            .iter_mut()
            .zip(self.multisig_signers.iter())
        {
            instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        // SAFETY: The expected number of accounts has been validated to be less than
        // the maximum allocated.
        unsafe {
            for (instruction_account, source) in instruction_accounts
                .get_unchecked_mut(3 + self.multisig_signers.len()..)
                .iter_mut()
                .zip(self.sources.iter())
            {
                instruction_account.write(InstructionAccount::writable(source.address()));
            }
        }

        // Accounts.

        let mut accounts =
            [const { MaybeUninit::<&AccountView>::uninit() }; MAX_STATIC_CPI_ACCOUNTS];

        accounts[0].write(self.mint);

        accounts[1].write(self.destination);

        accounts[2].write(self.authority);

        for (account, signer) in accounts[3..].iter_mut().zip(self.multisig_signers.iter()) {
            account.write(*signer);
        }

        // SAFETY: The expected number of accounts has been validated to be less than
        // the maximum allocated.
        unsafe {
            for (account, source) in accounts
                .get_unchecked_mut(3 + self.multisig_signers.len()..)
                .iter_mut()
                .zip(self.sources.iter())
            {
                account.write(*source);
            }
        }

        invoke_signed_with_bounds::<MAX_STATIC_CPI_ACCOUNTS>(
            &InstructionView {
                program_id: self.token_program,
                // SAFETY: instruction accounts has `expected_accounts` initialized.
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                data: &[
                    ExtensionDiscriminator::TransferFee as u8,
                    Self::DISCRIMINATOR,
                    self.sources.len() as u8,
                ],
            },
            // SAFETY: accounts has `expected_accounts` initialized.
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
