use {
    crate::{instructions::MAX_MULTISIG_SIGNERS, write_bytes, UNINIT_BYTE},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Approves a delegate.
///
/// ### Accounts:
///   * Single owner
///   0. `[WRITE]` The token account.
///   1. `[]` The delegate.
///   2. `[SIGNER]` The source account owner.
///
///   * Multisignature owner
///   0. `[WRITE]` The token account.
///   1. `[]` The delegate.
///   2. `[]` The source account's multisignature owner.
///   3. `..3+M` `[SIGNER]` M signer accounts
pub struct Approve<'a, 'b> {
    /// Source Account.
    pub source: &'a AccountView,
    /// Delegate Account
    pub delegate: &'a AccountView,
    /// Source Owner Account
    pub authority: &'a AccountView,
    /// Multisignature signers.
    pub multisig_signers: &'b [&'a AccountView],
    /// Amount
    pub amount: u64,
}

impl<'a, 'b> Approve<'a, 'b> {
    /// Creates a new `Approve` instruction with a single owner authority.
    #[inline(always)]
    pub fn new(
        source: &'a AccountView,
        delegate: &'a AccountView,
        authority: &'a AccountView,
        amount: u64,
    ) -> Self {
        Self::with_multisig_signers(source, delegate, authority, amount, &[])
    }

    /// Creates a new `Approve` instruction with a
    /// multisignature owner authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig_signers(
        source: &'a AccountView,
        delegate: &'a AccountView,
        authority: &'a AccountView,
        amount: u64,
        multisig_signers: &'b [&'a AccountView],
    ) -> Self {
        Self {
            source,
            delegate,
            authority,
            multisig_signers,
            amount,
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

        instruction_accounts[0].write(InstructionAccount::writable(self.source.address()));

        instruction_accounts[1].write(InstructionAccount::readonly(self.delegate.address()));

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

        accounts[0].write(self.source);

        accounts[1].write(self.delegate);

        accounts[2].write(self.authority);

        for (account, signer) in accounts[3..].iter_mut().zip(self.multisig_signers.iter()) {
            account.write(signer);
        }

        // Instruction data.
        // - [0]: instruction discriminator (1 byte, u8)
        // - [1..9]: amount (8 bytes, u64)

        let mut instruction_data = [UNINIT_BYTE; 9];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[4]);
        // Set amount as u64 at offset [1..9]
        write_bytes(&mut instruction_data[1..], &self.amount.to_le_bytes());

        invoke_signed_with_bounds::<{ 3 + MAX_MULTISIG_SIGNERS }>(
            &InstructionView {
                program_id: &crate::ID,
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 9) },
            },
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
