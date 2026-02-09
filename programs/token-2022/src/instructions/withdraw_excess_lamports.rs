use {
    crate::instructions::MAX_MULTISIG_SIGNERS,
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// This instruction is to be used to rescue SOL sent to any `TokenProgram`
/// owned account by sending them to any other account, leaving behind only
/// lamports for rent exemption.
///
/// 0. `[writable]` Source account owned by the token program.
/// 1. `[writable]` Destination account.
/// 2. `[signer]` Authority.
/// 3. ..`3+M` `[signer]` M signer accounts.
pub struct WidthdrawExcessLamports<'a, 'b, 'c> {
    /// Account to withdraw from.
    ///
    /// This account can be a mint, token, or multisig account.
    pub account: &'a AccountView,

    /// Destination account to receive the withdrawn lamports.
    pub destination: &'a AccountView,

    /// The owner/authority account.
    pub authority: &'a AccountView,

    /// Multisignature owner/authority.
    pub signers: &'c [&'a AccountView],

    /// Token Program
    pub token_program: &'b Address,
}

impl<'a, 'b, 'c> WidthdrawExcessLamports<'a, 'b, 'c> {
    pub const DISCRIMINATOR: u8 = 38;

    /// Creates a new `WidthdrawExcessLamports` instruction with a single
    /// owner/delegate authority.
    #[inline(always)]
    pub fn new(
        token_program: &'b Address,
        account: &'a AccountView,
        destination: &'a AccountView,
        authority: &'a AccountView,
    ) -> Self {
        Self::with_signers(token_program, account, destination, authority, &[])
    }

    /// Creates a new `WidthdrawExcessLamports` instruction with a
    /// multisignature owner/delegate authority and signer accounts.
    #[inline(always)]
    pub fn with_signers(
        token_program: &'b Address,
        account: &'a AccountView,
        destination: &'a AccountView,
        authority: &'a AccountView,
        signers: &'c [&'a AccountView],
    ) -> Self {
        Self {
            account,
            destination,
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
            Err(ProgramError::InvalidArgument)?;
        }

        let expected_accounts = 3 + self.signers.len();

        // Instruction accounts.

        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 3 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.account.address()));

            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::writable(self.destination.address()));

            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::new(
                    self.authority.address(),
                    false,
                    self.signers.is_empty(),
                ));

            for (account, signer) in instruction_accounts
                .get_unchecked_mut(3..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // Accounts.

        const UNINIT_INFO: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut accounts = [UNINIT_INFO; 3 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            accounts.get_unchecked_mut(0).write(self.account);

            accounts.get_unchecked_mut(1).write(self.destination);

            accounts.get_unchecked_mut(2).write(self.authority);

            for (account, signer) in accounts
                .get_unchecked_mut(3..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(signer);
            }
        }

        invoke_signed_with_bounds::<{ 3 + MAX_MULTISIG_SIGNERS }>(
            &InstructionView {
                program_id: self.token_program,
                // SAFETY: instruction accounts has `expected_accounts` initialized.
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                data: &[Self::DISCRIMINATOR],
            },
            // SAFETY: accounts has `expected_accounts` initialized.
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
