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

/// Mints tokens to confidential balance
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
pub struct Mint<'a, 'b, 'data> {
    /// The Token account
    pub token_account: &'a AccountView,
    /// The Token mint
    pub mint: &'a AccountView,
    /// The instruction sysvar
    pub intruction_sysvar: Option<&'a AccountView>,
    /// Context state for `VerifyCiphertextCommitmentEquality`
    /// proof
    pub commitment_equality_proof_context: Option<&'a AccountView>,
    /// Context state for `VerifyBatchedGroupedCiphertext3HandlesValidty`
    /// proof
    pub batched_group_validity_proof_context: Option<&'a AccountView>,
    /// Context state for `VerifyBatchedRangeProofU128` proof
    pub batched_range_proof_context: Option<&'a AccountView>,
    /// The authority
    pub authority: &'a AccountView,
    /// The multisig signers
    pub multisig_signers: &'b [&'a AccountView],
    /// The token program
    pub token_program: &'a Address,

    /// Data expected:
    ///
    /// The new `decryptable` supply if the mint succeeds
    pub new_decryptable_supply: &'data [u8; AE_CIPHERTEXT_LEN],
    /// The transfer amount encrypted under the auditor `ElGamal` public key
    pub mint_amount_auditor_ciphertext_lo: &'data [u8; ELGAMAL_CIPHERTEXT_LEN],
    /// The transfer amount encrypted under the auditor `ElGamal` public key
    pub mint_amount_auditor_ciphertext_hi: &'data [u8; ELGAMAL_CIPHERTEXT_LEN],
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

impl Mint<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 3;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signer_seeds: &[Signer]) -> ProgramResult {
        if self.multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        // instruction accounts

        let mut i = 0usize;

        // Cpi Accounts
        let mut accounts = [UNINIT_ACCOUNT_REF; 7 + MAX_MULTISIG_SIGNERS];

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 7 + MAX_MULTISIG_SIGNERS];

        // SAFETY: Allocations are valid to the maximum number of accounts
        unsafe {
            // token account
            instruction_accounts
                .get_unchecked_mut(i)
                .write(InstructionAccount::writable(self.token_account.address()));
            accounts.get_unchecked_mut(i).write(self.token_account);
            i += 1;

            // token mint
            instruction_accounts
                .get_unchecked_mut(i)
                .write(InstructionAccount::writable(self.mint.address()));
            accounts.get_unchecked_mut(i).write(self.mint);
            i += 1;

            // instruction sysvar if any `zk_elgamal_proof` program
            // instruction are included in the same transaction
            if let Some(instruction_sysvar_account) = self.intruction_sysvar {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        instruction_sysvar_account.address(),
                    ));
                accounts
                    .get_unchecked_mut(i)
                    .write(instruction_sysvar_account);
                i += 1;
            }

            // context state account for `VerifyCiphertextCommitmentEquality` proof
            if let Some(commitment_equality_proof_context_account) =
                self.commitment_equality_proof_context
            {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        commitment_equality_proof_context_account.address(),
                    ));
                accounts
                    .get_unchecked_mut(i)
                    .write(commitment_equality_proof_context_account);
                i += 1;
            }

            // context state account for `VerifyBatchedGroupedCiphertext3HandlesValidty`
            // proof
            if let Some(batched_group_validity_proof_context_account) =
                self.batched_group_validity_proof_context
            {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        batched_group_validity_proof_context_account.address(),
                    ));
                accounts
                    .get_unchecked_mut(i)
                    .write(batched_group_validity_proof_context_account);
                i += 1;
            }

            // context state account for `VerifyBatchedRangeProofU128` proof
            if let Some(batched_range_proof_context_account) = self.batched_range_proof_context {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        batched_range_proof_context_account.address(),
                    ));
                accounts
                    .get_unchecked_mut(i)
                    .write(batched_range_proof_context_account);
                i += 1;
            }

            // The mint authority
            instruction_accounts
                .get_unchecked_mut(i)
                .write(InstructionAccount::new(
                    self.authority.address(),
                    false,
                    self.multisig_signers.is_empty(),
                ));
            accounts.get_unchecked_mut(i).write(self.authority);
            i += 1;

            // the multisig signers
            for (account, signer) in instruction_accounts
                .get_unchecked_mut(i..)
                .iter_mut()
                .zip(self.multisig_signers.iter())
            {
                account.write(InstructionAccount::readonly_signer(signer.address()));
            }
            for (account, signer) in accounts
                .get_unchecked_mut(i..)
                .iter_mut()
                .zip(self.multisig_signers.iter())
            {
                account.write(*signer);
            }
        }

        // instruction data

        let mut instruction_data = [UNINIT_BYTE;
            2 + AE_CIPHERTEXT_LEN + ELGAMAL_CIPHERTEXT_LEN + ELGAMAL_CIPHERTEXT_LEN + 3];

        let mut offset = 0;

        // discriminators
        write_bytes(
            &mut instruction_data[offset..offset + 2],
            &[
                ExtensionDiscriminator::ConfidentialMintBurn as u8,
                Self::DISCRIMINATOR,
            ],
        );
        offset += 2;

        // new `decryptable` supply
        write_bytes(
            &mut instruction_data[offset..offset + AE_CIPHERTEXT_LEN],
            self.new_decryptable_supply.as_ref(),
        );
        offset += AE_CIPHERTEXT_LEN;

        // mint_amount_auditor_ciphertext_lo
        write_bytes(
            &mut instruction_data[offset..offset + ELGAMAL_CIPHERTEXT_LEN],
            self.mint_amount_auditor_ciphertext_lo.as_ref(),
        );
        offset += ELGAMAL_CIPHERTEXT_LEN;

        // mint_amount_auditor_ciphertext_hi
        write_bytes(
            &mut instruction_data[offset..offset + ELGAMAL_CIPHERTEXT_LEN],
            self.mint_amount_auditor_ciphertext_hi.as_ref(),
        );
        offset += ELGAMAL_CIPHERTEXT_LEN;

        // instruction offsets
        write_bytes(
            &mut instruction_data[offset..offset + 3],
            &[
                self.equality_proof_instruction_offset as u8,
                self.ciphertext_validity_proof_instruction_offset as u8,
                self.range_proof_instruction_offset as u8,
            ],
        );

        // instruction

        let expected_accounts = i + self.multisig_signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed_with_bounds::<{ 7 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signer_seeds,
        )
    }
}
