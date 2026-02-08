use {
    crate::{
        instructions::{ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
        write_bytes, UNINIT_ACCOUNT_REF, UNINIT_BYTE, UNINIT_INSTRUCTION_ACCOUNT,
    },
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Applies the pending balance to the available balance, based on the
/// history of `Deposit` and/or `Transfer` instructions.
///
/// After submitting `ApplyPendingBalance`, the client should compare
/// `ConfidentialTransferAccount::expected_pending_balance_credit_counter`
/// with
/// `ConfidentialTransferAccount::actual_applied_pending_balance_instructions`.
/// If they are equal then the
/// `ConfidentialTransferAccount::decryptable_available_balance` is
/// consistent with `ConfidentialTransferAccount::available_balance`. If
/// they differ then there is more pending balance to be applied.
///
/// Account expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The SPL Token account.
///   1. `[signer]` The single account owner.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The SPL Token account.
///   1. `[]` The multisig account owner.
///   2. .. `[signer]` Required M signer accounts for the SPL Token Multisig
pub struct ApplyPendingBalance<'a, 'b> {
    pub token_account: &'a AccountView,
    pub owner: &'a AccountView,
    pub multisig_signers: &'b [&'a AccountView],
    pub token_program: &'a Address,

    /// Data expected
    ///
    /// The expected number of pending balance credits since the last successful
    /// `ApplyPendingBalance` instruction
    pub expected_pending_balance_credit_counter: u64,
}

impl ApplyPendingBalance<'_, '_> {
    const DISCRIMINATOR: u8 = 8;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, singers: &[Signer]) -> ProgramResult {
        if self.multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // token account
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.token_account.address()));

            // owner
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::new(
                    self.owner.address(),
                    false,
                    self.multisig_signers.is_empty(),
                ));

            // multisig signers
            for (account, signer) in instruction_accounts
                .get_unchecked_mut(2..)
                .iter_mut()
                .zip(self.multisig_signers.iter())
            {
                account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // instruction data
        let mut instruction_data = [UNINIT_BYTE; 2 + 8];

        // discriminators
        write_bytes(
            &mut instruction_data[..2],
            &[
                ExtensionDiscriminator::ConfidentialTransfer as u8,
                ApplyPendingBalance::DISCRIMINATOR,
            ],
        );

        // expected pending balance credit counter
        write_bytes(
            &mut instruction_data[2..10],
            &self.expected_pending_balance_credit_counter.to_le_bytes(),
        );

        // instruction
        let expected_accounts = 2 + self.multisig_signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        // Cpi Accounts
        let mut accounts = [UNINIT_ACCOUNT_REF; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            //token account
            accounts.get_unchecked_mut(0).write(self.token_account);

            // token account owner
            accounts.get_unchecked_mut(1).write(self.owner);

            //multisig signers
            for (account, signer) in accounts
                .get_unchecked_mut(2..)
                .iter_mut()
                .zip(self.multisig_signers.iter())
            {
                account.write(*signer);
            }
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            singers,
        )
    }
}
