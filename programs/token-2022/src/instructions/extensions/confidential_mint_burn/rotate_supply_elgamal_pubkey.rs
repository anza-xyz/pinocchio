use {
    crate::{
        instructions::{ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
        write_bytes, ELGAMAL_PUBKEY_LEN, UNINIT_ACCOUNT_REF, UNINIT_BYTE,
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

/// Rotates the ElGamal pubkey used to encrypt confidential supply
///
/// The pending burn amount must be zero in order for this instruction
/// to be processed successfully.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The SPL Token mint.
///   1. `[]` Instructions sysvar if `CiphertextCiphertextEquality` is included
///      in the same transaction or context state account if
///      `CiphertextCiphertextEquality` is pre-verified into a context state
///      account.
///   2. `[signer]` Confidential mint authority.
///
///   * Multisignature authority
///   0. `[writable]` The SPL Token mint.
///   1. `[]` Instructions sysvar if `CiphertextCiphertextEquality` is included
///      in the same transaction or context state account if
///      `CiphertextCiphertextEquality` is pre-verified into a context state
///      account.
///   2. `[]` The multisig authority account owner.
///   3. ..`[signer]` Required M signer accounts for the SPL Token Multisig
pub struct RotateSupplyElgamalPubkey<'a, 'b, 'data> {
    /// The token mint
    pub mint: &'a AccountView,
    /// Instruction sysvar or Context State account for
    /// `CiphertextCiphertextEquality`
    pub instruction_sysvar_or_context_state: &'a AccountView,
    /// Confidential mint authority
    pub authority: &'a AccountView,
    /// Multisig signers if the authority is a multisig
    pub multisig_signers: &'b [&'a AccountView],
    /// The token program
    pub token_program: &'a Address,

    /// Data expected:
    ///
    /// The new ElGamal pubkey for supply encryption
    pub new_supply_elgamal_pubkey: &'data [u8; ELGAMAL_PUBKEY_LEN],
    /// The location of the
    /// `ProofInstruction::VerifyCiphertextCiphertextEquality` instruction
    /// relative to the `RotateSupplyElGamalPubkey` instruction in the
    /// transaction provide 0 for context state account
    pub proof_instruction_offset: i8,
}

impl RotateSupplyElgamalPubkey<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 1;

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

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 3 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // The token mint
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.mint.address()));

            // instruction sysvar or context state account
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly(
                    self.instruction_sysvar_or_context_state.address(),
                ));

            // The mint authority account
            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::new(
                    self.authority.address(),
                    false,
                    self.multisig_signers.is_empty(),
                ));

            // The multisig signers
            for (account, signer) in instruction_accounts
                .get_unchecked_mut(3..)
                .iter_mut()
                .zip(self.multisig_signers.iter())
            {
                account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // instruction data

        let mut instruction_data = [UNINIT_BYTE; 2 + ELGAMAL_PUBKEY_LEN + 1];

        // discriminators
        write_bytes(
            &mut instruction_data[..2],
            &[
                ExtensionDiscriminator::ConfidentialMintBurn as u8,
                Self::DISCRIMINATOR,
            ],
        );

        // new elgamal pubkey
        write_bytes(
            &mut instruction_data[3..3 + ELGAMAL_PUBKEY_LEN],
            self.new_supply_elgamal_pubkey.as_ref(),
        );

        // instruction offset
        instruction_data[34].write(self.proof_instruction_offset as u8);

        // instruction

        let expected_accounts = 3 + self.multisig_signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        // Accounts

        let mut accounts = [UNINIT_ACCOUNT_REF; 3 + MAX_MULTISIG_SIGNERS];

        // token mint
        accounts[0].write(self.mint);

        // instruction sysvar or context state
        accounts[1].write(self.instruction_sysvar_or_context_state);

        // confidential mint authority
        accounts[2].write(self.authority);

        for (account, signer) in accounts[3..].iter_mut().zip(self.multisig_signers.iter()) {
            account.write(*signer);
        }

        invoke_signed_with_bounds::<{ 3 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
