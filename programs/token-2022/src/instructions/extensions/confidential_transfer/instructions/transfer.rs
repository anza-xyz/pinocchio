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

/// Transfer tokens confidentially.
///
/// In order for this instruction to be successfully processed, it must be
/// accompanied by the following list of `zk_elgamal_proof` program
/// instructions:
///
/// - `VerifyCiphertextCommitmentEquality`
/// - `VerifyBatchedGroupedCiphertext3HandlesValidity`
/// - `VerifyBatchedRangeProofU128`
///
/// These instructions can be accompanied in the same transaction or can be
/// pre-verified into a context state account, in which case, only their
/// context state account addresses need to be provided.
///
/// Fails if the associated mint is extended as `NonTransferable`.
///
/// * Single owner/delegate
/// 1. `[writable]` The source SPL Token account.
/// 2. `[]` The token mint.
/// 3. `[writable]` The destination SPL Token account.
/// 4. `[]` (Optional) Instructions sysvar if at least one of the
///    `zk_elgamal_proof` instructions are included in the same transaction.
/// 5. `[]` (Optional) Equality proof context state account.
/// 6. `[]` (Optional) Cipher-text validity context state account.
/// 7. `[]` (Optional) Range proof context state account.
/// 8. `[signer]` The single source account owner.
///
/// * Multisignature owner/delegate
/// 1. `[writable]` The source SPL Token account.
/// 2. `[]` The token mint.
/// 3. `[writable]` The destination SPL Token account.
/// 4. `[]` (Optional) Instructions sysvar if at least one of the
///    `zk_elgamal_proof` instructions are included in the same transaction.
/// 5. `[]` (Optional) Equality proof context state account.
/// 6. `[]` (Optional) Cipher-text validity proof context state account.
/// 7. `[]` (Optional) Range proof context state account.
/// 8. `[]` The multisig  source account owner.
/// 9. .. `[signer]` Required M signer accounts for the SPL Token Multisig
pub struct Transfer<'a, 'b, 'data> {
    pub source_token_account: &'a AccountView,
    pub mint: &'a AccountView,
    pub destination_token_account: &'a AccountView,
    pub instruction_sysvar: Option<&'a AccountView>,
    pub equality_proof_context: Option<&'a AccountView>,
    pub ciphertext_proof_context: Option<&'a AccountView>,
    pub range_proof_context: Option<&'a AccountView>,
    pub source_owner: &'a AccountView,
    pub multisig_signers: &'b [&'a AccountView],
    pub token_program: &'a Address,

    /// Data expected
    ///
    /// The new source decrypt-able balance if the transfer succeeds
    pub new_source_decryptable_available_balance: &'data [u8; AE_CIPHERTEXT_LEN],
    /// The transfer amount encrypted under the auditor ElGamal public key
    pub transfer_amount_auditor_ciphertext_lo: &'data [u8; ELGAMAL_CIPHERTEXT_LEN],
    /// The transfer amount encrypted under the auditor ElGamal public key
    pub transfer_amount_auditor_ciphertext_hi: &'data [u8; ELGAMAL_CIPHERTEXT_LEN],
    /// instruction offsets of the proofs; provide 0 if the
    /// instruction is included in the same transaction
    pub equality_proof_instruction_offset: i8,
    pub ciphertext_proof_instruction_offset: i8,
    pub range_proof_instruction_offset: i8,
}

impl Transfer<'_, '_, '_> {
    const DISCRIMINATOR: u8 = 7;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        // insruction accounts

        let mut i = 0usize;

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 8 + MAX_MULTISIG_SIGNERS];

        // SAFETY: allocation is valid to the maximum number of accounts
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

            // instruction sysvar
            if let Some(instruction_sysvar_account) = self.instruction_sysvar {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        instruction_sysvar_account.address(),
                    ));
                i += 1;
            }

            // equality proof context
            if let Some(equality_proof_context_account) = self.equality_proof_context {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        equality_proof_context_account.address(),
                    ));
                i += 1;
            }

            // ciphertext validity proof context
            if let Some(ciphertext_proof_context_account) = self.ciphertext_proof_context {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        ciphertext_proof_context_account.address(),
                    ));
                i += 1;
            }

            // range proof context
            if let Some(range_proof_context_account) = self.range_proof_context {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        range_proof_context_account.address(),
                    ));
                i += 1;
            }

            // source token account owner
            instruction_accounts
                .get_unchecked_mut(i)
                .write(InstructionAccount::new(
                    self.source_owner.address(),
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

        let mut instruction_data = [UNINIT_BYTE; 2 + 36 + 64 + 64 + 1 + 1 + 1];

        // discriminators
        write_bytes(
            &mut instruction_data[..2],
            &[
                ExtensionDiscriminator::ConfidentialTransfer as u8,
                Transfer::DISCRIMINATOR,
            ],
        );

        // new source decrypt-able available balance
        write_bytes(
            &mut instruction_data[2..38],
            self.new_source_decryptable_available_balance,
        );

        // transfer amount auditor cipher-text lo
        write_bytes(
            &mut instruction_data[38..102],
            self.transfer_amount_auditor_ciphertext_lo,
        );

        // transfer amount auditor cipher-text hi
        write_bytes(
            &mut instruction_data[102..166],
            self.transfer_amount_auditor_ciphertext_hi,
        );

        // instruction offsets
        write_bytes(
            &mut instruction_data[166..169],
            &[
                self.equality_proof_instruction_offset as u8,
                self.ciphertext_proof_instruction_offset as u8,
                self.range_proof_instruction_offset as u8,
            ],
        );

        // CPI Instruction

        // non multisig signer accounts + multisig signers
        let expected_accounts = i + self.multisig_signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        // CPI Accounts

        let mut accounts = [UNINIT_ACCOUNT_REF; 8 + MAX_MULTISIG_SIGNERS];

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

            let mut i = 2usize;

            // instruction sysvar
            if let Some(instruction_sysvar_account) = self.instruction_sysvar {
                accounts
                    .get_unchecked_mut(i)
                    .write(instruction_sysvar_account);
                i += 1;
            }

            // equality proof context
            if let Some(equality_proof_context_account) = self.equality_proof_context {
                accounts
                    .get_unchecked_mut(i)
                    .write(equality_proof_context_account);
                i += 1;
            }

            // ciphertext validity proof context
            if let Some(ciphertext_proof_context_account) = self.ciphertext_proof_context {
                accounts
                    .get_unchecked_mut(i)
                    .write(ciphertext_proof_context_account);
                i += 1;
            }

            // range proof context
            if let Some(range_proof_context_account) = self.range_proof_context {
                accounts
                    .get_unchecked_mut(i)
                    .write(range_proof_context_account);
                i += 1;
            }

            // source token account owner
            accounts.get_unchecked_mut(i).write(self.source_owner);
            i += 1;

            // multisig signers
            for (account, signer) in accounts
                .get_unchecked_mut(i..)
                .iter_mut()
                .zip(self.multisig_signers.iter())
            {
                account.write(*signer);
            }
        }

        invoke_signed_with_bounds::<{ 8 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
