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

/// Configures confidential transfer for a token account
///
/// The instruction fails if the confidential transfers are already
/// configured, or if the mint was not initialized with confidential
/// transfer support.
///
/// The instruction fails if the `instructions::initialize_account`
/// instruction has not yet successfully executed for the token account.
///
/// Upon success, confidential and non-confidential deposits and transfers
/// are enabled. Use the `DisableConfidentialCredits` and
/// `DisableNonConfidentialCredits` instructions to disable.
///
/// In order for this instruction to be successfully processed, it must be
/// accompanied by the `VerifyPubkeyValidity` instruction of the
/// `zk_elgamal_proof` program in the same transaction.
///
/// Accounts expected by this instruction:
///
/// * Single owner/delegate
/// 0. `[writable]` The SPL Token account
/// 1. `[]` The corresponding SPL Token mint.
/// 2. `[]` Instruction sysvar if `VerifyPubkeyValidity` is included in the same
///    transaction or context state account if
///    `VerifyPubkeyValidity` is pre-verified into a context state
///    account.
/// 3. `[signer]` The single source account owner.
///
/// * Multisignature owner/delegate
/// 0. `[writable]` The SPL Token account
/// 1. `[]` The corresponding SPL Token mint.
/// 2. `[]` Instruction sysvar if `VerifyPubkeyValidity` is included in the same
///    transaction or context state account if
///    `VerifyPubkeyValidity` is pre-verified into a context state
///    account.
/// 3. `[]` The multisig source account owner.
/// 4. ..`[signer]` Required M signer accounts for the SPL Token Multisig
///    account.
pub struct ConfigureAccount<'a, 'b, 'data> {
    /// The Token account to be configured.
    pub token_account: &'a AccountView,
    /// The Token mint.
    pub mint: &'a AccountView,
    /// Instruction sysvar
    pub instruction_sysvar_or_context_state: &'a AccountView,
    /// The owner of the token account.
    pub authority: &'a AccountView,
    /// The signers if the authority is a multisig.
    pub signers: &'b [&'a AccountView],
    /// The token program
    pub token_program: &'a Address,

    /// Data expected by the instruction
    ///
    /// The maximum number of deposits and transfers that an account can receive
    /// before the `ApplyPendingBalance` is executed
    pub maximum_pending_balance_credit_counter: u64,
    /// The decrypt-able balance (always 0) once the configure account succeeds
    pub decryptable_zero_balance: &'data [u8; AE_CIPHERTEXT_LEN],
    /// Proof instruction offset
    /// provide 0 to use context state account
    pub proof_instruction_offset: i8,
}

impl ConfigureAccount<'_, '_, '_> {
    const DISCRIMINATOR: u8 = 2;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        // Instruction accounts
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 4 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            // token account
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.token_account.address()));

            // token mint
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly(self.mint.address()));

            // instruction sysvar or context state account
            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::readonly(
                    self.instruction_sysvar_or_context_state.address(),
                ));

            // owner/delegate
            instruction_accounts
                .get_unchecked_mut(3)
                .write(InstructionAccount::new(
                    self.authority.address(),
                    false,
                    self.signers.is_empty(),
                ));

            //multisig signers
            for (instruction_account, signer) in instruction_accounts
                .get_unchecked_mut(4..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // Instruction data
        // 2 (extension + insn discriminator) + 8 (u64) + 36 ([u8; 36]) + 1 (i8)
        let mut instruction_data = [UNINIT_BYTE; 2 + 45];

        // discriminators
        write_bytes(
            &mut instruction_data[..2],
            &[
                ExtensionDiscriminator::ConfidentialTransfer as u8,
                ConfigureAccount::DISCRIMINATOR,
            ],
        );

        // decryptable balance
        write_bytes(&mut instruction_data[2..38], self.decryptable_zero_balance);

        // maximum pending balance credit counter
        write_bytes(
            &mut instruction_data[38..46],
            &self.maximum_pending_balance_credit_counter.to_le_bytes(),
        );

        // proof instruction offset
        unsafe {
            instruction_data
                .get_unchecked_mut(46)
                .write(self.proof_instruction_offset as _);
        }

        // Instruction

        let expected_accounts = 4 + self.signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        // Accounts

        let mut accounts = [UNINIT_ACCOUNT_REF; 4 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            // token account
            accounts.get_unchecked_mut(0).write(self.token_account);

            // token mint
            accounts.get_unchecked_mut(1).write(self.mint);

            // instruction sysvar
            accounts.get_unchecked_mut(1).write(self.instruction_sysvar_or_context_state);

            // authority
            accounts.get_unchecked_mut(2).write(self.authority);

            // signers
            for (account, signer) in accounts
                .get_unchecked_mut(3..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(*signer);
            }
        }

        invoke_signed_with_bounds::<{ 4 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
