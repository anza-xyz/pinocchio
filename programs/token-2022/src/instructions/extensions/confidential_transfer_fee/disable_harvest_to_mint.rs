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

/// Configure a confidential transfer fee mint to reject any harvested
/// confidential fees.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The token mint.
///   1. `[signer]` The confidential transfer fee authority.
///
///   *Multisignature owner/delegate
///   0. `[writable]` The token mint.
///   1. `[]` The confidential transfer fee multisig authority,
///   2. `[signer]` Required M signer accounts for the SPL Token Multisig
///      account.
pub struct DisableHarvestToMint<'a, 'b> {
    /// The token mint
    pub mint: &'a AccountView,
    /// The confidential transfer fee authority
    pub authority: &'a AccountView,
    /// The multisig signers
    pub multisig_signers: &'b [&'a AccountView],
    /// The token program
    pub token_program: &'a Address,
}

impl DisableHarvestToMint<'_, '_> {
    pub const DISCRIMINATOR: u8 = 5;

    pub fn invoke_signed(&self, signers_seeds: &[Signer]) -> ProgramResult {
        if self.multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        };

        // Instruction accounts

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 2 + MAX_MULTISIG_SIGNERS];

        // Cpi Accounts

        let mut accounts = [UNINIT_ACCOUNT_REF; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY: Allocation is valid to the maximum number of accounts
        unsafe {
            // The token mint
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.mint.address()));
            accounts.get_unchecked_mut(0).write(self.mint);

            // The confidential transfer fee authority
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::new(
                    self.authority.address(),
                    false,
                    self.multisig_signers.is_empty(),
                ));
            accounts.get_unchecked_mut(1).write(self.authority);

            // The multisig signers
            for ((instruction_account, account), signer) in instruction_accounts
                .get_unchecked_mut(2..)
                .iter_mut()
                .zip(accounts.get_unchecked_mut(2..).iter_mut())
                .zip(self.multisig_signers.iter())
            {
                instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
                account.write(*signer);
            }
        };

        // instruction data
        let instruction_data = [
            ExtensionDiscriminator::ConfidentialTransferFee as u8,
            Self::DISCRIMINATOR,
        ];

        // instruction

        let expected_accounts = 2 + self.multisig_signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: instruction_data.as_ref(),
        };

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers_seeds,
        )
    }
}
