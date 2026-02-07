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
    solana_program_error::ProgramResult,
};

/// Withdraw SPL Tokens from the available balance of a confidential token
/// account.
///
/// In order for this instruction to be processed successfully, it must be
/// accompanied by the following list of `zk_elgamal_proof` program
/// instructions:
///
/// - `VerifyCiphertextCommitmentEquility`
/// - `VerifyBatchedRangeProofU64`
///
/// These instructions can be accompanied in the same transaction or can be
/// pre-verified into a context state account, in which case, only their
/// context state account address need to be provided.
///
/// Fails if the source or destination accounts are frozen.
/// Fails if the associated mint is extended as `NonTransferable`.
/// Fails if the associated mint is extended as `ConfidentialMintBurn`.
/// Fails if the associated mint is paused with the `Pausable` extension.
///
/// Accounts expected by this instruction:
///
/// * Single owner/delegate
/// 0. `[writable]` The SPL Token account.
/// 1. `[]` The token mint.
/// 2. `[]` (Optional) Instruction sysvar if at least one of the
///    `zk_elgamal_proof` instructions are included in the same transaction.
/// 3. `[]` (Optional) Equality proof context state account.
/// 4. `[]` (Optional) Range proof context state account.
/// 5. `[signer]` The single source account owner.
///
/// * Multisignature owner/delegate
/// 0. `[writable]` The SPL Token account.
/// 1. `[]` The token mint.
/// 2. `[]` (Optional) Instructions sysvar if at least one of the
///    `zk_elgamal_proof` instructions are included in the same transaction.
/// 3. `[]` (Optional) Equality proof context state account.
/// 4. `[]` (Optional) Range proof context state account.
/// 5. `[]` The multisig source account owner.
/// 6. ...`[signer]` Required M signer accounts for the SPL Token Multisig
///    account.
pub struct Withdraw<'a, 'b> {
    /// The Token account
    pub token_account: &'a AccountView,
    /// The SPL Token mint
    pub mint: &'a AccountView,
    /// Instruction sysvar if any `zk_elgamal_proof` instructions are
    /// included in the same transaction.
    pub instruction_sysvar: Option<&'a AccountView>,
    /// Equality proof context state, if pre-verified
    pub equality_proof_context: Option<&'a AccountView>,
    /// Range proof context state, if pre-verified
    pub range_proof_context: Option<&'a AccountView>,
    /// The token account owner
    pub owner: &'a AccountView,
    /// The multisig signers
    pub signers: &'b [&'a AccountView],
    /// token program
    pub token_program: &'a Address,

    /// Data expected
    ///
    /// The amount of tokens to withdraw
    pub amount: u64,
    /// The expected number of base 10 digits to the right of the decimal place
    pub decimals: u8,
    /// Relative location of the
    /// `ProofInstruction::VerifyCiphertextCommitmentEquality` instruction
    /// to the `Withdraw` instruction in the transaction. If the offset is
    /// `0`, then use a context state account for the proof.
    pub equality_proof_instruction_offset: i8,
    /// Relative location of the `ProofInstruction::BatchedRangeProofU64`
    /// instruction to the `Withdraw` instruction in the transaction. If the
    /// offset is `0`, then use a context state account for the proof.
    pub range_proof_instruction_offset: i8,
}

impl Withdraw<'_, '_> {
    const DISCRIMINATOR: u8 = 6;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(solana_program_error::ProgramError::InvalidArgument);
        }

        // Instruction Accounts

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 6 + MAX_MULTISIG_SIGNERS];

        // SAFETY: allocation is valid to the maximum number of accounts
        unsafe {
            let mut i = 0usize;
            // token account
            instruction_accounts
                .get_unchecked_mut(i)
                .write(InstructionAccount::writable(self.token_account.address()));

            // next
            i += 1;

            // token mint
            instruction_accounts
                .get_unchecked_mut(i)
                .write(InstructionAccount::readonly(self.mint.address()));

            // next
            i += 1;

            // instruction sysvar
            if let Some(instruction_sysvar_account) = self.instruction_sysvar {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        instruction_sysvar_account.address(),
                    ));
            }

            //next
            i += 1;

            // equality proof context state account
            if let Some(equality_proof_context_account) = self.equality_proof_context {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        equality_proof_context_account.address(),
                    ));
            }

            // next
            i += 1;

            // range proof context state account
            if let Some(range_proof_context_account) = self.range_proof_context {
                instruction_accounts
                    .get_unchecked_mut(i)
                    .write(InstructionAccount::readonly(
                        range_proof_context_account.address(),
                    ));
            }

            // next
            i += 1;

            // owner
            instruction_accounts
                .get_unchecked_mut(i)
                .write(InstructionAccount::new(
                    self.owner.address(),
                    false,
                    self.signers.is_empty(),
                ));

            // next
            i += 1;

            // multisig signers
            for (account, signer) in instruction_accounts
                .get_unchecked_mut(i..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // instruction data
        let mut instruction_data = [UNINIT_BYTE; 2 + 8 + 1 + 1 + 1];

        // discriminators
        write_bytes(
            &mut instruction_data[..2],
            &[
                ExtensionDiscriminator::ConfidentialTransfer as u8,
                Withdraw::DISCRIMINATOR,
            ],
        );

        // amount
        write_bytes(&mut instruction_data[2..10], &self.amount.to_le_bytes());

        unsafe {
            // decimals
            instruction_data.get_unchecked_mut(10).write(self.decimals);

            // equalit proof instruction offset
            instruction_data
                .get_unchecked_mut(11)
                .write(self.equality_proof_instruction_offset as u8);

            // range proof instruction offset
            instruction_data
                .get_unchecked_mut(12)
                .write(self.range_proof_instruction_offset as u8);
        }

        // instruction

        let expected_accounts = 6 + self.signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        // Accounts

        let mut accounts = [UNINIT_ACCOUNT_REF; 6 + MAX_MULTISIG_SIGNERS];

        unsafe {
            let mut i = 0usize;

            // token account
            accounts.get_unchecked_mut(i).write(self.token_account);
            i += 1;

            // token  mint
            accounts.get_unchecked_mut(i).write(self.mint);
            i += 1;

            // instruction sysvar
            if let Some(instruction_sysvar_account) = self.instruction_sysvar {
                accounts
                    .get_unchecked_mut(i)
                    .write(instruction_sysvar_account);
                i += 1;
            }

            // equality proof context state
            if let Some(equality_context_account) = self.equality_proof_context {
                accounts
                    .get_unchecked_mut(i)
                    .write(equality_context_account);
                i += 1;
            }

            // range proof instruction offset
            if let Some(range_proof_context_account) = self.range_proof_context {
                accounts
                    .get_unchecked_mut(i)
                    .write(range_proof_context_account);
                i += 1;
            }

            // owner
            accounts.get_unchecked_mut(i).write(self.owner);
            i += 1;

            // multisig signers
            for (account, signer) in accounts.get_unchecked_mut(i..).iter_mut().zip(self.signers) {
                account.write(*signer);
            }
        }

        invoke_signed_with_bounds::<{ 6 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
