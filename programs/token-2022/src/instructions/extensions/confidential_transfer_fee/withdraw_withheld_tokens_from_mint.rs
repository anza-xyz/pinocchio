use {
    crate::{
        instructions::{ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
        write_bytes, AE_CIPHERTEXT_LEN, UNINIT_ACCOUNT_REF, UNINIT_BYTE,
        UNINIT_INSTRUCTION_ACCOUNT,
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

/// Transfer all withheld confidential tokens in the mint to an account.
/// Signed by the mint's withdraw withheld tokens authority.
///
/// The withheld confidential tokens are aggregated directly into the
/// destination available balance.
///
/// In order for this instruction to be successfully processed, it must be
/// accompanied by the `VerifyCiphertextCiphertextEquality` instruction
/// of the `zk_elgamal_proof` program in the same transaction or the
/// address of a context state account for the proof must be provided.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The token mint. Must include the `TransferFeeConfig`
///      extension.
///   1. `[writable]` The fee receiver account. Must include the
///      `TransferFeeAmount` and `ConfidentialTransferAccount` extensions.
///   2. `[]` Instructions sysvar if `VerifyCiphertextCiphertextEquality` is
///      included in the same transaction or context state account if
///      `VerifyCiphertextCiphertextEquality` is pre-verified into a context
///      state account.
///   3. `[signer]` The mint's `withdraw_withheld_authority`.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The token mint. Must include the `TransferFeeConfig`
///      extension.
///   1. `[writable]` The fee receiver account. Must include the
///      `TransferFeeAmount` and `ConfidentialTransferAccount` extensions.
///   2. `[]` Instructions sysvar if `VerifyCiphertextCiphertextEquality` is
///      included in the same transaction or context state account if
///      `VerifyCiphertextCiphertextEquality` is pre-verified into a context
///      state account.
///   3. `[]` The mint's multisig `withdraw_withheld_authority`.
///   4. ..`4+M` `[signer]` M signer accounts.
pub struct WithdrawWithheldTokensFromMint<'a, 'b, 'data> {
    /// The token mint
    pub mint: &'a AccountView,
    /// The fee receiver account
    pub receiver_account: &'a AccountView,
    /// The instruction sysvar or context state account
    pub instruction_sysvar_or_context_state: &'a AccountView,
    /// The mint's `withdraw_withheld_authority`
    pub withdraw_withheld_authority: &'a AccountView,
    /// The multisig signers
    pub multisig_signers: &'b [&'a AccountView],
    /// The token program
    pub token_program: &'a Address,

    /// Data expected:
    ///
    /// Relative location of the `VerifyCiphertextCiphertextEquality`
    /// instruction to the `WithdrawWithheldTokensFromMint` instruction in
    /// the transaction. If the offset is `0`, then use a context state
    /// account for the proof.
    pub proof_instruction_offset: i8,
    /// The new `decryptable` balance in the destination token account.
    pub new_decryptable_available_balance: &'data [u8; AE_CIPHERTEXT_LEN],
}

impl WithdrawWithheldTokensFromMint<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 1;

    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers_seeds: &[Signer]) -> ProgramResult {
        if self.multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        };

        // Instruction accounts and Cpi Accounts

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 4 + MAX_MULTISIG_SIGNERS];
        let mut accounts = [UNINIT_ACCOUNT_REF; 4 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            // The token mint
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.mint.address()));
            accounts.get_unchecked_mut(0).write(self.mint);

            // The receiver token account
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::writable(
                    self.receiver_account.address(),
                ));
            accounts.get_unchecked_mut(1).write(self.receiver_account);

            // The instruction sysvar or context state account for
            // `VerifyCiphertextCiphertextEquality`
            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::readonly(
                    self.instruction_sysvar_or_context_state.address(),
                ));
            accounts
                .get_unchecked_mut(2)
                .write(self.instruction_sysvar_or_context_state);

            // The mint's withdraw withheld authority
            instruction_accounts
                .get_unchecked_mut(3)
                .write(InstructionAccount::new(
                    self.withdraw_withheld_authority.address(),
                    false,
                    self.multisig_signers.is_empty(),
                ));
            accounts
                .get_unchecked_mut(3)
                .write(self.withdraw_withheld_authority);

            // The multisig signers
            for ((insn_account, account), signer) in instruction_accounts
                .get_unchecked_mut(4..)
                .iter_mut()
                .zip(accounts.get_unchecked_mut(4..).iter_mut())
                .zip(self.multisig_signers.iter())
            {
                insn_account.write(InstructionAccount::readonly_signer(signer.address()));
                account.write(*signer);
            }
        }

        // instruction data

        let mut instruction_data = [UNINIT_BYTE; 2 + 1 + AE_CIPHERTEXT_LEN];

        // discrminators
        write_bytes(
            &mut instruction_data[..2],
            &[
                ExtensionDiscriminator::ConfidentialTransferFee as u8,
                Self::DISCRIMINATOR,
            ],
        );

        unsafe {
            // instruction offset
            instruction_data
                .get_unchecked_mut(2)
                .write(self.proof_instruction_offset as u8);
        }

        // new `Decryptable` available balance
        write_bytes(
            &mut instruction_data[3..3 + AE_CIPHERTEXT_LEN],
            self.new_decryptable_available_balance.as_ref(),
        );

        // instruction

        let expected_accounts = 4 + self.multisig_signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed_with_bounds::<{ 4 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers_seeds,
        )
    }
}
