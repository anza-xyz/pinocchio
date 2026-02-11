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

/// Transfer tokens confidentially with fee.
///
/// In order for this instruction to be successfully processed, it must be
/// accompanied by the following list of `zk_elgamal_proof` program
/// instructions:
///
/// - `VerifyCiphertextCommitmentEquality`
/// - `VerifyBatchedGroupedCiphertext3HandlesValidity` (transfer amount
///   cipher-text)
/// - `VerifyPercentageWithFee`
/// - `VerifyBatchedGroupedCiphertext2HandlesValidity` (fee cipher-text)
/// - `VerifyBatchedRangeProofU256`
///
/// These instructions can be accompanied in the same transaction or can be
/// pre-verified into a context state account, in which case, only their
/// context state account addresses need to be provided.
///
/// The same restrictions for the `Transfer` applies to
/// `TransferWithFee`. Namely, the instruction fails if the
/// associated mint is extended as `NonTransferable`.
///
///   * Transfer without fee
///   1. `[writable]` The source SPL Token account.
///   2. `[]` The token mint.
///   3. `[writable]` The destination SPL Token account.
///   4. `[]` (Optional) Instructions sysvar if at least one of the
///      `zk_elgamal_proof` instructions are included in the same transaction.
///   5. `[]` (Optional) Equality proof context state account.
///   6. `[]` (Optional) Transfer amount cipher-text validity proof context
///      state account.
///   7. `[]` (Optional) Fee sigma proof context state account.
///   8. `[]` (Optional) Fee cipher-text validity proof context state account.
///   9. `[]` (Optional) Range proof context state account.
///   10. `[signer]` The source account owner.
///
///   * Transfer with fee
///   1. `[writable]` The source SPL Token account.
///   2. `[]` The token mint.
///   3. `[writable]` The destination SPL Token account.
///   4. `[]` (Optional) Instructions sysvar if at least one of the
///      `zk_elgamal_proof` instructions are included in the same transaction.
///   5. `[]` (Optional) Equality proof context state account.
///   6. `[]` (Optional) Transfer amount cipher-text validity proof context
///      state account.
///   7. `[]` (Optional) Fee sigma proof context state account.
///   8. `[]` (Optional) Fee cipher-text validity proof context state account.
///   9. `[]` (Optional) Range proof context state account.
///   10. `[]` The multisig  source account owner.
///   11. .. `[signer]` Required M signer accounts for the SPL Token Multisig
pub struct TransferWithFee<'a, 'b, 'data> {
    pub source_token_account: &'a AccountView,
    pub mint: &'a AccountView,
    pub destination_token_account: &'a AccountView,
    pub instruction_sysvar: Option<&'a AccountView>,
    pub equality_proof_context: Option<&'a AccountView>,
    pub amount_ciphertext_proof_context: Option<&'a AccountView>,
    pub fee_sigma_proof_context: Option<&'a AccountView>,
    pub fee_ciphertext_proof_context: Option<&'a AccountView>,
    pub range_proof_context: Option<&'a AccountView>,
    pub owner: &'a AccountView,
    pub multisig_signers: &'b [&'a AccountView],
    pub token_program: &'a Address,

    /// Data expected
    ///
    /// The new source decryptable balance if the transfer succeeds
    pub new_source_decryptable_available_balance: &'data [u8; AE_CIPHERTEXT_LEN],
    /// The transfer amount encrypted under the auditor ElGamal public key
    pub transfer_amount_auditor_ciphertext_lo: &'data [u8; ELGAMAL_CIPHERTEXT_LEN],
    /// The transfer amount encrypted under the auditor ElGamal public key
    pub transfer_amount_auditor_ciphertext_hi: &'data [u8; ELGAMAL_CIPHERTEXT_LEN],
    /// Relative location of the `proof instruction` to the `TransferWithFee`
    /// instruction in the transaction.
    /// If the offset is `0`, then use a context state account for the
    /// proof.
    pub equality_proof_instruction_offset: i8,
    pub amount_ciphertext_proof_instruction_offset: i8,
    pub fee_sigma_proof_instruction_offset: i8,
    pub fee_ciphertext_proof_instruction_offset: i8,
    pub range_proof_instruction_offset: i8,
}

impl<'a, 'b, 'data> TransferWithFee<'a, 'b, 'data> {
    const DISCRIMINATOR: u8 = 13;

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

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 10 + MAX_MULTISIG_SIGNERS];

        let mut i: usize = 0;

        // SAFETY: account allocation valid upto the maximum expected accounts
        unsafe {
            // source token account
            instruction_accounts
                .get_unchecked_mut(i)
                .write(InstructionAccount::writable(
                    self.source_token_account.address(),
                ));
            i += 1;

            // token mint
            instruction_accounts
                .get_unchecked_mut(i)
                .write(InstructionAccount::readonly(self.mint.address()));
            i += 1;

            // destination token account
            instruction_accounts
                .get_unchecked_mut(i)
                .write(InstructionAccount::writable(
                    self.destination_token_account.address(),
                ));
            i += 1;

            // instruction sysvar if provided
            if let Some(instruction_sysvar_account) = self.instruction_sysvar {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        instruction_sysvar_account.address(),
                    ));
                i += 1;
            }

            // equality proof context state account
            if let Some(equality_proof_context_account) = self.equality_proof_context {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        equality_proof_context_account.address(),
                    ));
                i += 1;
            }

            // transfer amount cipher-text validity proof context state account
            if let Some(amount_ciphertext_proof_context_account) =
                self.amount_ciphertext_proof_context
            {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        amount_ciphertext_proof_context_account.address(),
                    ));
                i += 1;
            }

            // fee sigma proof context state account
            if let Some(fee_sigma_proof_context_account) = self.fee_sigma_proof_context {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        fee_sigma_proof_context_account.address(),
                    ));
                i += 1;
            }

            // fee cipher-text validity proof context state account
            if let Some(fee_ciphertext_proof_context_account) = self.fee_ciphertext_proof_context {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        fee_ciphertext_proof_context_account.address(),
                    ));
                i += 1;
            }

            // range proof context state account
            if let Some(range_proof_context_account) = self.range_proof_context {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        range_proof_context_account.address(),
                    ));
                i += 1;
            }

            // token account authority - owner/delegate
            instruction_accounts
                .get_unchecked_mut(i)
                .write(InstructionAccount::new(
                    self.owner.address(),
                    false,
                    self.multisig_signers.is_empty(),
                ));
            i += 1;

            // multisig signers
            for (account, signer) in instruction_accounts
                .get_unchecked_mut(i..)
                .iter_mut()
                .zip(self.multisig_signers.iter())
            {
                account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // instruction data

        let mut instruction_data = [UNINIT_BYTE;
            2 + AE_CIPHERTEXT_LEN + ELGAMAL_CIPHERTEXT_LEN + ELGAMAL_CIPHERTEXT_LEN + 1 + 1 + 1];

        let mut offset: usize = 0;

        // extension discriminator + extension instruction discriminator
        write_bytes(
            &mut instruction_data[offset..offset + 2],
            &[
                ExtensionDiscriminator::ConfidentialTransfer as u8,
                Self::DISCRIMINATOR,
            ],
        );
        offset += 2;

        // new source decrypt-able available balance
        write_bytes(
            &mut instruction_data[offset..offset + AE_CIPHERTEXT_LEN],
            self.new_source_decryptable_available_balance,
        );
        offset += AE_CIPHERTEXT_LEN;

        // transfer amount auditor cipher-text lo
        write_bytes(
            &mut instruction_data[offset..offset + ELGAMAL_CIPHERTEXT_LEN],
            self.transfer_amount_auditor_ciphertext_lo,
        );
        offset += ELGAMAL_CIPHERTEXT_LEN;

        // transfer amount auditor cipher-text hi
        write_bytes(
            &mut instruction_data[offset..offset + ELGAMAL_CIPHERTEXT_LEN],
            self.transfer_amount_auditor_ciphertext_hi,
        );
        offset += ELGAMAL_CIPHERTEXT_LEN;

        // instruction offset
        write_bytes(
            &mut instruction_data[offset..offset + 5],
            &[
                self.equality_proof_instruction_offset as u8,
                self.amount_ciphertext_proof_instruction_offset as u8,
                self.fee_sigma_proof_instruction_offset as u8,
                self.fee_ciphertext_proof_instruction_offset as u8,
                self.range_proof_instruction_offset as u8,
            ],
        );

        // Instruction

        // expected accounts = non multisig signer accounts + multisig signers
        let expected_accounts = i + self.multisig_signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        // Accounts

        let mut accounts = [UNINIT_ACCOUNT_REF; 10 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // source token account
            accounts
                .get_unchecked_mut(0)
                .write(self.source_token_account);

            // token mint
            accounts.get_unchecked_mut(1).write(self.mint);

            // destination token account
            accounts
                .get_unchecked_mut(2)
                .write(self.destination_token_account);

            let mut i = 3usize;

            // instruction sysvar if provided
            if let Some(instruction_sysvar_account) = self.instruction_sysvar {
                accounts
                    .get_unchecked_mut(i)
                    .write(instruction_sysvar_account);
                i += 1;
            }

            // equality proof context state account
            if let Some(equality_proof_context_account) = self.equality_proof_context {
                accounts
                    .get_unchecked_mut(i)
                    .write(equality_proof_context_account);
                i += 1;
            }

            // transfer amount cipher-text validity proof context state account
            if let Some(amount_ciphertext_proof_context_account) =
                self.amount_ciphertext_proof_context
            {
                accounts
                    .get_unchecked_mut(i)
                    .write(amount_ciphertext_proof_context_account);
                i += 1;
            }

            // fee sigma proof context state account
            if let Some(fee_sigma_proof_context_account) = self.fee_sigma_proof_context {
                accounts
                    .get_unchecked_mut(i)
                    .write(fee_sigma_proof_context_account);
                i += 1;
            }

            // fee cipher-text validity proof context state account
            if let Some(fee_ciphertext_proof_context_account) = self.fee_ciphertext_proof_context {
                accounts
                    .get_unchecked_mut(i)
                    .write(fee_ciphertext_proof_context_account);
                i += 1;
            }

            // range proof context state account
            if let Some(range_proof_context_account) = self.range_proof_context {
                accounts
                    .get_unchecked_mut(i)
                    .write(range_proof_context_account);
                i += 1;
            }

            // token account authority - owner/delegate
            accounts.get_unchecked_mut(i).write(self.owner);
            i += 1;

            // multisig signers
            for (account, signer) in accounts
                .get_unchecked_mut(i..)
                .iter_mut()
                .zip(self.multisig_signers.iter())
            {
                account.write(signer);
            }
        }

        invoke_signed_with_bounds::<{ 10 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
