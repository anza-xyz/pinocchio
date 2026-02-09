use {
    crate::{
        instructions::{extensions::ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
        write_bytes, UNINIT_BYTE,
    },
    core::{
        mem::MaybeUninit,
        slice::{self, from_raw_parts},
    },
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Burn tokens when the mint has the permissioned burn extension enabled.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The source account to burn from.
///   1. `[writable]` The token mint.
///   2. `[signer]` The permissioned burn authority configured on the mint, if
///      any.
///   3. `[signer]` The source account's owner/delegate.
///
///   * Multisignature authority
///   0. `[writable]` The source account to burn from.
///   1. `[writable]` The token mint.
///   2. `[signer]` The permissioned burn authority configured on the mint, if
///      any.
///   3. `[]` The source account's multisignature owner/delegate.
///   4. `..4+M` `[signer]` M signer accounts for the multisig.
pub struct Burn<'a, 'b, 'c> {
    /// The source account to burn from.
    pub account: &'a AccountView,

    /// The token mint.
    pub mint: &'a AccountView,

    /// The permissioned burn authority configured on the mint, if any.
    pub permissioned_burn_authority: &'a AccountView,

    /// The source account's owner/delegate.
    pub authority: &'a AccountView,

    /// Signer accounts for multisignature authority, if applicable.
    pub signers: &'c [&'a AccountView],

    /// The amount of tokens to burn.
    pub amount: u64,

    /// The token program.
    pub token_program: &'b Address,
}

impl<'a, 'b, 'c> Burn<'a, 'b, 'c> {
    pub const DISCRIMINATOR: u8 = 1;

    /// Creates a new `Burn` instruction with a single owner/delegate
    /// authority.
    #[inline(always)]
    pub fn new(
        token_program: &'b Address,
        account: &'a AccountView,
        mint: &'a AccountView,
        permissioned_burn_authority: &'a AccountView,
        authority: &'a AccountView,
        amount: u64,
    ) -> Self {
        Self::with_signers(
            token_program,
            account,
            mint,
            permissioned_burn_authority,
            authority,
            amount,
            &[],
        )
    }

    /// Creates a new `Burn` instruction with a multisignature owner/delegate
    /// authority and signer accounts.
    #[inline(always)]
    pub fn with_signers(
        token_program: &'b Address,
        account: &'a AccountView,
        mint: &'a AccountView,
        permissioned_burn_authority: &'a AccountView,
        authority: &'a AccountView,
        amount: u64,
        signers: &'c [&'a AccountView],
    ) -> Self {
        Self {
            account,
            mint,
            permissioned_burn_authority,
            authority,
            signers,
            amount,
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

        let expected_accounts = 4 + self.signers.len();

        // Instruction accounts.

        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 4 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.account.address()));

            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::writable(self.mint.address()));

            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::readonly_signer(
                    self.permissioned_burn_authority.address(),
                ));

            instruction_accounts
                .get_unchecked_mut(3)
                .write(InstructionAccount::new(
                    self.authority.address(),
                    false,
                    self.signers.is_empty(),
                ));

            for (account, signer) in instruction_accounts
                .get_unchecked_mut(4..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // Accounts.

        const UNINIT_INFO: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut accounts = [UNINIT_INFO; 4 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            accounts.get_unchecked_mut(0).write(self.account);

            accounts.get_unchecked_mut(1).write(self.mint);

            accounts
                .get_unchecked_mut(2)
                .write(self.permissioned_burn_authority);

            accounts.get_unchecked_mut(3).write(self.authority);

            for (account, signer) in accounts
                .get_unchecked_mut(4..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(signer);
            }
        }

        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 10];

        // discriminators
        instruction_data[0].write(ExtensionDiscriminator::PermissionedBurn as u8);
        instruction_data[1].write(Self::DISCRIMINATOR);
        // amount
        write_bytes(&mut instruction_data[2..10], &self.amount.to_le_bytes());

        invoke_signed_with_bounds::<{ 4 + MAX_MULTISIG_SIGNERS }>(
            &InstructionView {
                program_id: self.token_program,
                // SAFETY: instruction accounts has `expected_accounts` initialized.
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                // SAFETY: `instruction_data` is initialized.
                data: unsafe {
                    from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len())
                },
            },
            // SAFETY: accounts has `expected_accounts` initialized.
            unsafe { slice::from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
