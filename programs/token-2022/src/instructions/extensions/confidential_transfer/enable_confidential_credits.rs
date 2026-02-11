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

/// Configure a confidential extension account to accept incoming
/// confidential transfers.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The SPL Token account.
///   1. `[signer]` Single authority.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The SPL Token account.
///   1. `[]` Multisig authority.
///   2. .. `[signer]` Required M signer accounts for the SPL Token Multisig
pub struct EnableConfidentialCredits<'a, 'b> {
    pub token_account: &'a AccountView,
    pub owner: &'a AccountView,
    pub multisig_signers: &'b [&'a AccountView],
    pub token_program: &'a Address,
}

impl EnableConfidentialCredits<'_, '_> {
    const DISCRIMINATOR: u8 = 9;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        // instruction accounts
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // token account
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.token_account.address()));

            // token account owner
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount {
                    address: self.owner.address(),
                    is_writable: false,
                    is_signer: self.multisig_signers.is_empty(),
                });

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
        let instruction_data = [
            ExtensionDiscriminator::ConfidentialTransfer as u8,
            EnableConfidentialCredits::DISCRIMINATOR,
        ];

        // Instruction

        let expected_accounts = 2 + self.multisig_signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: &instruction_data,
        };

        // Cpi accounts
        let mut accounts = [UNINIT_ACCOUNT_REF; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // token account
            accounts.get_unchecked_mut(0).write(self.token_account);

            // token account owner
            accounts.get_unchecked_mut(1).write(self.owner);

            // multisig signers
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
            signers,
        )
    }
}
