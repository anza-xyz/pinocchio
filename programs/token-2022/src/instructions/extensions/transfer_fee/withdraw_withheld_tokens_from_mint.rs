use {
    crate::{
        instructions::{ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
        UNINIT_ACCOUNT_REF, UNINIT_INSTRUCTION_ACCOUNT,
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

/// Transfer all withheld tokens in the mint to an account. Signed by the
/// mint's withdraw withheld tokens authority.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The token mint. Must include the `TransferFeeConfig`
///      extension.
///   1. `[writable]` The fee receiver account. Must include the
///      `TransferFeeAmount` extension associated with the provided mint.
///   2. `[signer]` The mint's `withdraw_withheld_authority`.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The token mint.
///   1. `[writable]` The destination account.
///   2. `[]` The mint's multisig `withdraw_withheld_authority`.
///   3. `..3+M` `[signer]` M signer accounts.
pub struct WithdrawWithheldTokensFromMint<'a, 'b, 'c> {
    /// The token mint.
    pub mint: &'a AccountView,

    /// The fee receiver account.
    pub destination: &'a AccountView,

    /// The mint's `withdraw_withheld_authority` or multisig.
    pub authority: &'a AccountView,

    /// Multisignature owner/delegate.
    pub signers: &'c [&'a AccountView],

    /// Token program.
    pub token_program: &'b Address,
}

impl WithdrawWithheldTokensFromMint<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 2;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        // Instruction accounts.

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 3 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            // mint account
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.mint.address()));

            // destination account
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::writable(self.destination.address()));

            // authority account (single or multisig)
            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::new(
                    self.authority.address(),
                    false,
                    self.signers.is_empty(),
                ));

            // multisig signer accounts
            for (instruction_account, signer) in instruction_accounts
                .get_unchecked_mut(3..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // Instruction.

        let expected_accounts = 4 + self.signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: &[
                ExtensionDiscriminator::TransferFee as u8,
                Self::DISCRIMINATOR,
            ],
        };

        // Accounts.

        let mut accounts = [UNINIT_ACCOUNT_REF; 3 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            // mint account
            accounts.get_unchecked_mut(0).write(self.mint);

            // destination account
            accounts.get_unchecked_mut(1).write(self.destination);

            // authority account (single or multisig)
            accounts.get_unchecked_mut(2).write(self.authority);

            // multisig signer accounts
            for (account, signer) in accounts
                .get_unchecked_mut(3..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(*signer);
            }
        }

        invoke_signed_with_bounds::<{ 3 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
