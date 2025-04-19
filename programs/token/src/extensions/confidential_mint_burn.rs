use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    ProgramResult,
};

use crate::TOKEN_2022_PROGRAM_ID;

use super::{
    confidential_transfer::{
        DecryptableBalance, EncryptedBalance, PodAeCiphertext, PodElGamalCiphertext,
    },
    get_extension_from_bytes, PodElGamalPubkey,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct ConfidentialMintBurn {
    /// The confidential supply of the mint (encrypted by `encryption_pubkey`)
    pub confidential_supply: PodElGamalCiphertext,
    /// The decryptable confidential supply of the mint
    pub decryptable_supply: PodAeCiphertext,
    /// The ElGamal pubkey used to encrypt the confidential supply
    pub supply_elgamal_pubkey: PodElGamalPubkey,
    /// The amount of burn amounts not yet aggregated into the confidential supply
    pub pending_burn: PodElGamalCiphertext,
}

impl super::Extension for ConfidentialMintBurn {
    const TYPE: super::ExtensionType = super::ExtensionType::ConfidentialMintBurn;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl ConfidentialMintBurn {
    /// The length of the `ConfidentialMintBurn` account data.
    pub const LEN: usize = core::mem::size_of::<ConfidentialMintBurn>();

    /// Return a `ConfidentialMintBurn` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&ConfidentialMintBurn, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(ProgramError::InvalidAccountData)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SupplyAccountInfo {
    /// The available balance (encrypted by `supply_elgamal_pubkey`)
    pub current_supply: PodElGamalCiphertext,
    /// The decryptable supply
    pub decryptable_supply: PodAeCiphertext,
    /// The supply's ElGamal pubkey
    pub supply_elgamal_pubkey: PodElGamalPubkey,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BurnAccountInfo {
    /// The available balance (encrypted by `encryption_pubkey`)
    pub available_balance: EncryptedBalance,
    /// The decryptable available balance
    pub decryptable_available_balance: DecryptableBalance,
}

// Instructions
pub struct InitializeMintData<'a> {
    /// The mint to initialize
    pub mint: &'a AccountInfo,
    /// The ElGamal pubkey used to encrypt the confidential supply
    pub supply_elgamal_pubkey: PodElGamalPubkey,
    /// The initial 0 supply encrypted with the supply aes key
    pub decryptable_supply: PodAeCiphertext,
}

impl InitializeMintData<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        Ok(())
    }
}

pub struct RotateSupplyElGamalPubkey<'a> {
    /// The mint to rotate
    pub mint: &'a AccountInfo,
    /// Instruction sysvar
    pub instruction_sysvar: &'a AccountInfo,
    /// The confidential mint authority
    pub confidential_mint_authority: &'a AccountInfo,
    /// The new ElGamal pubkey for supply encryption
    pub new_supply_elgamal_pubkey: PodElGamalPubkey,
    /// The location of the
    /// `ProofInstruction::VerifyCiphertextCiphertextEquality` instruction
    /// relative to the `RotateSupplyElGamal` instruction in the transaction
    pub proof_instruction_offset: i8,
}

impl RotateSupplyElGamalPubkey<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        Ok(())
    }
}

pub struct UpdateDecryptableSupply<'a> {
    /// The mint to update
    pub mint: &'a AccountInfo,
    /// The confidential mint authority
    pub confidential_mint_authority: &'a AccountInfo,
    /// The new decryptable supply
    pub new_decryptable_supply: PodAeCiphertext,
}

impl UpdateDecryptableSupply<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        Ok(())
    }
}

pub struct Mint<'a> {
    /// THe token account to mint to
    pub account: &'a AccountInfo,
    /// The mint to mint from
    pub mint: &'a AccountInfo,
    /// The instruction sysvar
    pub instruction_sysvar: &'a AccountInfo,
    /// Verify ciphertext commitment equality
    pub verify_ciphertext_commitment_equality: &'a AccountInfo,
    /// Verify batched grouped ciphertext handles validity
    pub verify_batched_grouped_cihertext3_handles_validity: &'a AccountInfo,
    /// Verify batched range proof u128
    pub verify_batched_range_proof_u128: &'a AccountInfo,
    /// The token account's owner
    pub account_owner: &'a AccountInfo,
    /// The new decryptable supply if the mint succeeds
    pub new_decryptable_supply: PodAeCiphertext,
    /// The transfer amount encrypted under the auditor ElGamal public key
    pub mint_amount_auditor_ciphertext_lo: PodElGamalCiphertext,
    /// The transfer amount encrypted under the auditor ElGamal public key
    pub mint_amount_auditor_ciphertext_hi: PodElGamalCiphertext,
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

impl Mint<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        Ok(())
    }
}

pub struct Burn<'a> {
    /// The SPL Token account to burn from
    pub account: &'a AccountInfo,
    /// The SPL Token mint
    pub mint: &'a AccountInfo,
    /// (Optional) Instructions sysvar if at least one of the `zk_elgamal_proof` instructions is included in the same transaction
    pub instruction_sysvar: Option<&'a AccountInfo>,
    /// (Optional) The context state account containing the pre-verified `VerifyCiphertextCommitmentEquality` proof
    pub verify_ciphertext_commitment_equality: Option<&'a AccountInfo>,
    /// (Optional) The context state account containing the pre-verified `VerifyBatchedGroupedCiphertext3HandlesValidity` proof
    pub verify_batched_grouped_ciphertext3_handles_validity: Option<&'a AccountInfo>,
    /// (Optional) The context state account containing the pre-verified `VerifyBatchedRangeProofU128`
    pub verify_batched_range_proof_u128: Option<&'a AccountInfo>,
    /// The single account owner
    pub account_owner: &'a AccountInfo,
    /// The new decryptable balance of the burner if the burn succeeds
    pub new_decryptable_available_balance: DecryptableBalance,
    /// The transfer amount encrypted under the auditor ElGamal public key
    pub burn_amount_auditor_ciphertext_lo: PodElGamalCiphertext,
    /// The transfer amount encrypted under the auditor ElGamal public key
    pub burn_amount_auditor_ciphertext_hi: PodElGamalCiphertext,
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

impl Burn<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        Ok(())
    }
}

pub struct ApplyPendingBurn<'a> {
    /// The SPL Token mint
    pub mint: &'a AccountInfo,
    /// The mint's authority
    pub mint_authority: &'a AccountInfo,
}

impl ApplyPendingBurn<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.mint_authority.key()),
        ];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[42, 5],
        };

        invoke_signed(&instruction, &[self.mint, self.mint_authority], signers)?;
        Ok(())
    }
}
