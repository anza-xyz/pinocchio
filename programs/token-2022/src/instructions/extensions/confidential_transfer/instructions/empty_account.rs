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

/// Empty the available balance in a confidential token account.
///
/// A token account that is extended for confidential transfers can only be
/// closed if the pending and available balance ciphertexts are emptied.
/// The pending balance can be emptied
/// via the `ConfidentialTransferInstruction::ApplyPendingBalance`
/// instruction. Use the `ConfidentialTransferInstruction::EmptyAccount`
/// instruction to empty the available balance ciphertext.
///
/// Note that a newly configured account is always empty, so this
/// instruction is not required prior to account closing if no
/// instructions beyond
/// `ConfidentialTransferInstruction::ConfigureAccount` have affected the
/// token account.
///
/// In order for this instruction to be successfully processed, it must be
/// accompanied by the `VerifyZeroCiphertext` instruction of the
/// `zk_elgamal_proof` program in the same transaction or the address of a
/// context state account for the proof must be provided.
///
/// * Single owner/delegate
/// 0. `[writable]` The SPL Token account.
/// 1. `[]` Instructions sysvar if `VerifyZeroCiphertext` is included in the
///    same transaction or context state account if `VerifyZeroCiphertext` is
///    pre-verified into a context state account.
/// 2. `[signer]` The single account owner.
///
/// * Multisignature owner/delegate
/// 0. `[writable]` The SPL Token account.
/// 1. `[]` Instructions sysvar if `VerifyZeroCiphertext` is included in the
///    same transaction or context state account if `VerifyZeroCiphertext` is
///    pre-verified into a context state account.
/// 2. `[]` The multisig account owner.
/// 3. .. `[signer]` Required M signer accounts for the SPL Token Multisig
///    account.
pub struct EmptyAccount<'a, 'b> {
    /// The Token account to be emptied
    pub token_account: &'a AccountView,
    /// Instruction Sysvar or context state account
    pub instruction_sysvar_or_context_state: &'a AccountView,
    /// The token account owner/delegate
    pub owner: &'a AccountView,
    /// The multisig signers
    pub signers: &'b [&'a AccountView],
    /// token program
    pub token_program: &'a Address,
    /// instruction offset
    /// 0, for context state account
    pub instruction_offset: i8,
}

impl EmptyAccount<'_, '_> {
    const DISCRIMINATOR: u8 = 4;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        // instruction accounts

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 3 + MAX_MULTISIG_SIGNERS];

        // SAFETY: allocation is valid to the maximum number of accounts
        unsafe {
            // token account
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.token_account.address()));

            // instruction sysvar or context state account
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly(
                    self.instruction_sysvar_or_context_state.address(),
                ));

            // authority of the token account
            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::new(
                    self.owner.address(),
                    false,
                    self.signers.is_empty(),
                ));

            // multisig signers
            for (account, signer) in instruction_accounts
                .get_unchecked_mut(3..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // instruction data

        // discriminators (2) + instruction offset (1)
        let mut instruction_data = [UNINIT_BYTE; 3];

        // discriminators
        write_bytes(
            &mut instruction_data[..2],
            &[
                ExtensionDiscriminator::ConfidentialTransfer as u8,
                EmptyAccount::DISCRIMINATOR,
            ],
        );

        // instruction offset
        unsafe {
            instruction_data
                .get_unchecked_mut(2)
                .write(self.instruction_offset as u8);
        }

        // Instruction

        let expected_accounts = 3 + self.signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        // Accounts

        let mut accounts = [UNINIT_ACCOUNT_REF; 3 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // token account
            accounts.get_unchecked_mut(0).write(self.token_account);

            // instruction sysvar or context state account
            accounts
                .get_unchecked_mut(1)
                .write(self.instruction_sysvar_or_context_state);

            // token account owner
            accounts.get_unchecked_mut(2).write(self.owner);

            //multisig signers
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
