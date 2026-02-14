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
        cpi::{invoke_signed_with_bounds, Signer, MAX_STATIC_CPI_ACCOUNTS},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Transfer all withheld tokens to an account. Signed by the mint's
/// withdraw withheld tokens authority. This instruction is susceptible
/// to front-running. Use `HarvestWithheldTokensToMint` and
/// `WithdrawWithheldTokensFromMint` as an alternative.
///
/// The withheld confidential tokens are aggregated directly into the
/// destination available balance.
///
/// Note on front-running: This instruction requires a zero-knowledge proof
/// verification instruction that is checked with respect to the account
/// state (the currently withheld fees). Suppose that a withdraw
/// withheld authority generates the
/// `WithdrawWithheldTokensFromAccounts` instruction along with a
/// corresponding zero-knowledge proof for a specified set of accounts,
/// and submits it on chain. If the withheld fees at any
/// of the specified accounts change before the
/// `WithdrawWithheldTokensFromAccounts` is executed on chain, the
/// zero-knowledge proof will not verify with respect to the new state,
/// forcing the transaction to fail.
///
/// If front-running occurs, then users can look up the updated states of
/// the accounts, generate a new zero-knowledge proof and try again.
/// Alternatively, withdraw withheld authority can first move the
/// withheld amount to the mint using `HarvestWithheldTokensToMint` and
/// then move the withheld fees from mint to a specified destination
/// account using `WithdrawWithheldTokensFromMint`.
///
/// In order for this instruction to be successfully processed, it must be
/// accompanied by the `VerifyWithdrawWithheldTokens` instruction of the
/// `zk_elgamal_proof` program in the same transaction or the address of a
/// context state account for the proof must be provided.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[]` The token mint. Must include the `TransferFeeConfig` extension.
///   1. `[writable]` The fee receiver account. Must include the
///      `TransferFeeAmount` and `ConfidentialTransferAccount` extensions.
///   2. `[]` Instructions sysvar if `VerifyCiphertextCiphertextEquality` is
///      included in the same transaction or context state account if
///      `VerifyCiphertextCiphertextEquality` is pre-verified into a context
///      state account.
///   3. `[signer]` The mint's `withdraw_withheld_authority`.
///   4. ..`4+N` `[writable]` The source accounts to withdraw from.
///
///   * Multisignature owner/delegate
///   0. `[]` The token mint. Must include the `TransferFeeConfig` extension.
///   1. `[writable]` The fee receiver account. Must include the
///      `TransferFeeAmount` and `ConfidentialTransferAccount` extensions.
///   2. `[]` Instructions sysvar if `VerifyCiphertextCiphertextEquality` is
///      included in the same transaction or context state account if
///      `VerifyCiphertextCiphertextEquality` is pre-verified into a context
///      state account.
///   3. `[]` The mint's multisig `withdraw_withheld_authority`.
///   4. ..`4+M` `[signer]` M signer accounts.
///   5. `5+M+1..5+M+N` `[writable]` The source accounts to withdraw from.
pub struct WithdrawWithheldTokensFromAccounts<'a, 'b, 'data> {
    /// The token mint
    pub mint: &'a AccountView,
    /// The fee receiver account
    pub receiver_account: &'a AccountView,
    /// The instruction sysvar or context state
    pub instruction_sysvar_or_context_state: &'a AccountView,
    /// The `withdraw_withheld_authority`
    pub withdraw_withheld_authority: &'a AccountView,
    /// The multisig signers
    pub multisig_signers: &'b [&'a AccountView],
    /// The source accounts to withdraw from
    pub source_accounts: &'b [&'a AccountView],
    /// The token program
    pub token_program: &'a Address,

    /// Data expected:
    ///
    /// Relative location of the `ProofInstruction::VerifyWithdrawWithheld`
    /// instruction to the `VerifyWithdrawWithheldTokensFromAccounts`
    /// instruction in the transaction. If the offset is `0`, then use a
    /// context state account for the proof.
    pub proof_instruction_offset: i8,
    /// The new `decryptable` balance in the destination token account.
    pub new_decryptable_available_balance: &'data [u8; AE_CIPHERTEXT_LEN],
}

impl WithdrawWithheldTokensFromAccounts<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 2;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers_seeds: &[Signer]) -> ProgramResult {
        if self.multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        };

        let expected_accounts = 4 + self.multisig_signers.len() + self.source_accounts.len();

        if expected_accounts > MAX_STATIC_CPI_ACCOUNTS {
            return Err(ProgramError::InvalidArgument);
        };

        // Instructions Accounts & Cpi Accounts

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; MAX_STATIC_CPI_ACCOUNTS];

        let mut accounts = [UNINIT_ACCOUNT_REF; MAX_STATIC_CPI_ACCOUNTS];

        // SAFETY: The expected number of accounts has been validated to be less than
        // the maximum allocated.
        unsafe {
            // The token mint
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::readonly(self.mint.address()));
            accounts.get_unchecked_mut(0).write(self.mint);

            // The fee receiver account
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::writable(
                    self.receiver_account.address(),
                ));
            accounts.get_unchecked_mut(1).write(self.receiver_account);

            // The instruction sysvar or context state account
            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::readonly(
                    self.instruction_sysvar_or_context_state.address(),
                ));
            accounts
                .get_unchecked_mut(2)
                .write(self.instruction_sysvar_or_context_state);

            // The withdraw withheld authority account
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

            // The multisig signer accounts
            for ((instruction_account, account), signer) in instruction_accounts
                .get_unchecked_mut(4..)
                .iter_mut()
                .zip(accounts.get_unchecked_mut(4..).iter_mut())
                .zip(self.multisig_signers.iter())
            {
                instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
                account.write(*signer);
            }
        }

        // instruction data

        let mut instruction_data = [UNINIT_BYTE; 2 + 1 + 1 + AE_CIPHERTEXT_LEN];

        // Extension discrminators + Instruction discrminator
        write_bytes(
            &mut instruction_data[..2],
            &[
                ExtensionDiscriminator::ConfidentialTransferFee as u8,
                Self::DISCRIMINATOR,
            ],
        );

        unsafe {
            // num of token accounts
            instruction_data
                .get_unchecked_mut(2)
                .write(self.source_accounts.len() as u8);

            // proof instruction offset
            instruction_data
                .get_unchecked_mut(3)
                .write(self.proof_instruction_offset as u8);
        }

        // new decryptable available balance
        write_bytes(
            &mut instruction_data[4..4 + AE_CIPHERTEXT_LEN],
            self.new_decryptable_available_balance,
        );

        // instruction
        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed_with_bounds::<{ MAX_STATIC_CPI_ACCOUNTS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers_seeds,
        )
    }
}
