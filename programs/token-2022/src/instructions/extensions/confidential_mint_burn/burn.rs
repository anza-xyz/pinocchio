use {
    crate::{
        instructions::{ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
        write_bytes, AE_CIPHERTEXT_LEN, ELGAMAL_CIPHERTEXT_LEN, UNINIT_ACCOUNT_REF, UNINIT_BYTE,
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

/// Burn tokens from confidential balance
///
/// Fails if the destination account is frozen.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The SPL Token account.
///   1. `[writable]` The SPL Token mint.
///   2. `[]` (Optional) Instructions sysvar if at least one of the
///      `zk_elgamal_proof` instructions are included in the same transaction.
///   3. `[]` (Optional) The context state account containing the pre-verified
///      `VerifyCiphertextCommitmentEquality` proof
///   4. `[]` (Optional) The context state account containing the pre-verified
///      `VerifyBatchedGroupedCiphertext3HandlesValidity` proof
///   5. `[]` (Optional) The context state account containing the pre-verified
///      `VerifyBatchedRangeProofU128`
///   6. `[signer]` The single account owner.
///
///   * Multisignature authority
///   0. `[writable]` The SPL Token account.
///   1. `[]` The SPL Token mint. `[writable]` if the mint has a non-zero supply
///      elgamal-pubkey
///   2. `[]` (Optional) Instructions sysvar if at least one of the
///      `zk_elgamal_proof` instructions are included in the same transaction.
///   3. `[]` (Optional) The context state account containing the pre-verified
///      `VerifyCiphertextCommitmentEquality` proof
///   4. `[]` (Optional) The context state account containing the pre-verified
///      `VerifyBatchedGroupedCiphertext3HandlesValidity` proof
///   5. `[]` (Optional) The context state account containing the pre-verified
///      `VerifyBatchedRangeProofU128`
///   6. `[]` The multisig account owner.
///   7. ..`[signer]` Required M signer accounts for the SPL Token Multisig
pub struct Burn<'a, 'b, 'data> {
    /// The token account
    pub token_account: &'a AccountView,
    /// The Token mint
    pub mint: &'a AccountView,
    /// instruction sysvar
    pub instruction_sysvar: Option<&'a AccountView>,
    /// The context state account for `VerifyCiphertextCommitmentEquality`
    /// proof
    pub commitment_equality_proof_context: Option<&'a AccountView>,
    /// The context state account for
    /// `VerifyBatchedGroupedCiphertext3HandlesValidity` proof
    pub batched_grouped_validity_proof_context: Option<&'a AccountView>,
    /// The context state account for `VerifyBatchedRangeProofU128` proof
    pub batch_range_proof_context: Option<&'a AccountView>,
    /// The token account owner
    pub owner: &'a AccountView,
    /// The multisig signers
    pub multisig_signers: &'b [&'a AccountView],
    /// The token program
    pub token_program: &'a Address,

    /// Data expected:
    ///
    /// The new `decryptable` balance of the burner if the burn succeeds
    pub new_decryptable_available_balance: &'data [u8; AE_CIPHERTEXT_LEN],
    /// The transfer amount encrypted under the auditor `ElGamal` public key
    pub burn_amount_auditor_ciphertext_lo: &'data [u8; ELGAMAL_CIPHERTEXT_LEN],
    /// The transfer amount encrypted under the auditor `ElGamal` public key
    pub burn_amount_auditor_ciphertext_hi: &'data [u8; ELGAMAL_CIPHERTEXT_LEN],
    /// Relative location of the
    /// `ProofInstruction::VerifyCiphertextCommitmentEquality` instruction
    /// to the `ConfidentialMint` instruction in the transaction. 0 if the
    /// proof is in a pre-verified context account
    pub equality_proof_instruction_offset: i8,
    /// Relative location of the
    /// `ProofInstruction::VerifyBatchedGroupedCiphertext3HandlesValidity`
    /// instruction to the `ConfidentialMint` instruction in the
    /// transaction. 0 if the proof is in a pre-verified context account
    pub ciphertext_validity_proof_instruction_offset: i8,
    /// Relative location of the `ProofInstruction::VerifyBatchedRangeProofU128`
    /// instruction to the `ConfidentialMint` instruction in the
    /// transaction. 0 if the proof is in a pre-verified context account
    pub range_proof_instruction_offset: i8,
}

impl Burn<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 4;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers_seeds: &[Signer]) -> ProgramResult {
        if self.multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        // instruction accounts

        let mut i = 0usize;

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 7 + MAX_MULTISIG_SIGNERS];

        // Cpi Accounts
        let mut accounts = [UNINIT_ACCOUNT_REF; 7 + MAX_MULTISIG_SIGNERS];

        // token account
        instruction_accounts[i].write(InstructionAccount::writable(self.token_account.address()));
        accounts[i].write(self.token_account);
        i += 1;

        // token mint
        instruction_accounts[i].write(InstructionAccount::writable(self.mint.address()));
        accounts[i].write(self.mint);
        i += 1;

        // instruction sysvar
        if let Some(instruction_sysvar_account) = self.instruction_sysvar {
            instruction_accounts[i].write(InstructionAccount::readonly(
                instruction_sysvar_account.address(),
            ));
            accounts[i].write(instruction_sysvar_account);
            i += 1;
        };

        // context state account for `VerifyCiphertextCommitmentEquality`
        if let Some(commitment_equality_proof_context_account) =
            self.commitment_equality_proof_context
        {
            instruction_accounts[i].write(InstructionAccount::readonly(
                commitment_equality_proof_context_account.address(),
            ));
            accounts[i].write(commitment_equality_proof_context_account);
            i += 1;
        };

        // context state account for `VerifyBatchedGroupedCiphertext3HandlesValidity`
        if let Some(batched_grouped_validity_proof_context_account) =
            self.batched_grouped_validity_proof_context
        {
            instruction_accounts[i].write(InstructionAccount::readonly(
                batched_grouped_validity_proof_context_account.address(),
            ));
            accounts[i].write(batched_grouped_validity_proof_context_account);
            i += 1;
        };

        // context state account for `VerifyBatchedRangeProofU128`
        if let Some(batched_range_proof_context_account) = self.batch_range_proof_context {
            instruction_accounts[i].write(InstructionAccount::readonly(
                batched_range_proof_context_account.address(),
            ));
            accounts[i].write(batched_range_proof_context_account);
            i += 1;
        };

        // The token account owner
        instruction_accounts[i].write(InstructionAccount::new(
            self.owner.address(),
            false,
            self.multisig_signers.is_empty(),
        ));
        accounts[i].write(self.owner);
        i += 1;

        // The multisig signers
        for ((instruction_account, account), signer) in instruction_accounts[i..]
            .iter_mut()
            .zip(accounts[i..].iter_mut())
            .zip(self.multisig_signers.iter())
        {
            instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
            account.write(*signer);
        }

        // instruction data

        let mut instruction_data = [UNINIT_BYTE;
            2 + AE_CIPHERTEXT_LEN + ELGAMAL_CIPHERTEXT_LEN + ELGAMAL_CIPHERTEXT_LEN + 3];

        let mut offset = 0usize;

        // discriminators
        write_bytes(
            &mut instruction_data[offset..2],
            &[
                ExtensionDiscriminator::ConfidentialMintBurn as u8,
                Self::DISCRIMINATOR,
            ],
        );
        offset += 2;

        // new `decryptable` available balance
        write_bytes(
            &mut instruction_data[offset..offset + AE_CIPHERTEXT_LEN],
            self.new_decryptable_available_balance.as_ref(),
        );
        offset += AE_CIPHERTEXT_LEN;

        // burnt amount auditor `ciphertext` lo
        write_bytes(
            &mut instruction_data[offset..offset + ELGAMAL_CIPHERTEXT_LEN],
            self.burn_amount_auditor_ciphertext_lo,
        );
        offset += ELGAMAL_CIPHERTEXT_LEN;

        // burnt amount auditor `ciphertext` hi
        write_bytes(
            &mut instruction_data[offset..offset + ELGAMAL_CIPHERTEXT_LEN],
            self.burn_amount_auditor_ciphertext_hi,
        );
        offset += ELGAMAL_CIPHERTEXT_LEN;

        // instrution offsets for `VerififyCiphertextCommitmentEquality`
        write_bytes(
            &mut instruction_data[offset..offset + 3],
            &[
                self.equality_proof_instruction_offset as u8,
                self.ciphertext_validity_proof_instruction_offset as u8,
                self.range_proof_instruction_offset as u8,
            ],
        );

        // instruction

        let expected_account = i + self.multisig_signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_account)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed_with_bounds::<{ 7 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_account) },
            signers_seeds,
        )
    }
}
