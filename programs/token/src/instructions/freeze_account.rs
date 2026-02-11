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

/// Freeze an initialized account using the Mint's freeze authority.
///
/// ### Accounts:
///   * Single freeze authority
///   0. `[WRITE]` The account to freeze.
///   1. `[]` The token mint.
///   2. `[SIGNER]` The mint freeze authority.
///
///   * Multisignature freeze authority
///   0. `[WRITE]` The account to freeze.
///   1. `[]` The token mint.
///   2. `[]` The mint's multisignature freeze authority.
///   3. ..3+M `[SIGNER]` M signer accounts
pub struct FreezeAccount<'a, 'b> {
    /// Token Account to freeze.
    pub account: &'a AccountView,
    /// Mint Account.
    pub mint: &'a AccountView,
    /// Mint Freeze Authority Account
    pub freeze_authority: &'a AccountView,
    /// Multisignature signers.
    pub multisig_signers: &'b [&'a AccountView],
}

impl<'a, 'b> FreezeAccount<'a, 'b> {
    /// Creates a new `FreezeAccount` instruction with a single freeze authority.
    #[inline(always)]
    pub fn new(
        account: &'a AccountView,
        mint: &'a AccountView,
        freeze_authority: &'a AccountView,
    ) -> Self {
        Self::with_multisig_signers(account, mint, freeze_authority, &[])
    }

    /// Creates a new `FreezeAccount` instruction with a
    /// multisignature freeze authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig_signers(
        account: &'a AccountView,
        mint: &'a AccountView,
        freeze_authority: &'a AccountView,
        multisig_signers: &'b [&'a AccountView],
    ) -> Self {
        Self {
            account,
            mint,
            freeze_authority,
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

        instruction_accounts[1].write(InstructionAccount::readonly(self.mint.address()));

        instruction_accounts[2].write(InstructionAccount::new(
            self.freeze_authority.address(),
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

        accounts[1].write(self.mint);

        accounts[2].write(self.freeze_authority);

        for (account, signer) in accounts[3..].iter_mut().zip(self.multisig_signers.iter()) {
            account.write(signer);
        }

        invoke_signed_with_bounds::<{ 3 + MAX_MULTISIG_SIGNERS }>(
            &InstructionView {
                program_id: &crate::ID,
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                data: &[10],
            },
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
