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

/// Applies the pending burn amount to the confidential supply
///
///   * Single authority
///   0. `[writable]` The SPL token mint.
///   1. `[signer]` The single mint authority.
///
///   * Multisignature authority
///   0. `[writable]` The SPL token mint.
///   1. `[]` The multisig account owner.
///   2. .. `[signer]` Required M signer accounts for the SPL Token Multisig
///      account.
pub struct ApplyPendingBurn<'a, 'b> {
    /// The token mint
    pub mint: &'a AccountView,
    /// The mint authority
    pub authority: &'a AccountView,
    /// The multisig signers
    pub multisig_signers: &'b [&'a AccountView],
    /// The token program
    pub token_program: &'a Address,
}

impl ApplyPendingBurn<'_, '_> {
    pub const DISCRIMINATOR: u8 = 5;

    pub fn invoke_signed(&self, signers_seeds: &[Signer]) -> ProgramResult {
        if self.multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        };

        // instruction accounts

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 2 + MAX_MULTISIG_SIGNERS];

        // the token mint
        instruction_accounts[0].write(InstructionAccount::writable(self.mint.address()));

        // the token mint authority
        instruction_accounts[1].write(InstructionAccount {
            address: self.authority.address(),
            is_writable: false,
            is_signer: self.multisig_signers.is_empty(),
        });

        // the multisig signers
        for (account, signer) in instruction_accounts[2..]
            .iter_mut()
            .zip(self.multisig_signers.iter())
        {
            account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        // instruction data
        let instruction_data = [
            ExtensionDiscriminator::ConfidentialMintBurn as u8,
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

        // Cpi Accounts
        let mut accounts = [UNINIT_ACCOUNT_REF; 2 + MAX_MULTISIG_SIGNERS];

        // token mint
        accounts[0].write(self.mint);

        // token mint authority
        accounts[1].write(self.authority);

        // the multisig signers
        for (account, signer) in accounts[2..].iter_mut().zip(self.multisig_signers.iter()) {
            account.write(*signer);
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers_seeds,
        )
    }
}
