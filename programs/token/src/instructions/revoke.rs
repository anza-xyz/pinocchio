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

/// Revokes the delegate's authority.
///
/// ### Accounts:
///   * Single owner
///   0. `[WRITE]` The source account.
///   1. `[SIGNER]` The source account owner.
///
///   * Multisignature owner
///   0. `[WRITE]` The source account.
///   1. `[]` The source account's multisignature owner.
///   2. `..2+M` `[SIGNER]` M signer accounts
pub struct Revoke<'a, 'b> {
    /// Source Account.
    pub source: &'a AccountView,
    ///  Source Owner Account.
    pub authority: &'a AccountView,
    /// Multisignature signers.
    pub multisig_signers: &'b [&'a AccountView],
}

impl<'a, 'b> Revoke<'a, 'b> {
    /// Creates a new `Revoke` instruction with a single owner authority.
    #[inline(always)]
    pub fn new(source: &'a AccountView, authority: &'a AccountView) -> Self {
        Self::with_multisig_signers(source, authority, &[])
    }

    /// Creates a new `Revoke` instruction with a
    /// multisignature owner authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig_signers(
        source: &'a AccountView,
        authority: &'a AccountView,
        multisig_signers: &'b [&'a AccountView],
    ) -> Self {
        Self {
            source,
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

        let expected_accounts = 2 + self.multisig_signers.len();

        // Instruction accounts.

        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; 2 + MAX_MULTISIG_SIGNERS];

        instruction_accounts[0].write(InstructionAccount::writable(self.source.address()));

        instruction_accounts[1].write(InstructionAccount::new(
            self.authority.address(),
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

        accounts[0].write(self.source);

        accounts[1].write(self.authority);

        for (account, signer) in accounts[2..].iter_mut().zip(self.multisig_signers.iter()) {
            account.write(signer);
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &InstructionView {
                program_id: &crate::ID,
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                data: &[5],
            },
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
