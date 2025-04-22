use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    ProgramResult,
};

use crate::{write_bytes, UNINIT_BYTE};

use super::{
    get_extension_from_bytes, DecryptableBalance, EncryptedBalance, PodAeCiphertext,
    PodElGamalCiphertext, PodElGamalPubkey, ELGAMAL_PUBKEY_LEN, POD_AE_CIPHERTEXT_LEN,
    POD_ELGAMAL_CIPHERTEXT_LEN,
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
        if !account_info.is_owned_by(&crate::ID) {
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
    pub supply_elgamal_pubkey: [u8; ELGAMAL_PUBKEY_LEN],
    /// The initial 0 supply encrypted with the supply aes key
    pub decryptable_supply: [u8; POD_AE_CIPHERTEXT_LEN],
}

impl InitializeMintData<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [AccountMeta::writable(self.mint.key())];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [2..34]: supply ElGamal pubkey (32 bytes, [u8;ELGAMAL_PUBKEY_LEN])
        // -  [34..70]: initial decryptable supply (36 bytes, [u8; POD_AE_CIPHERTEXT_LEN])

        let mut instruction_data = [UNINIT_BYTE; 70];
        // Set discriminator as u8 at offset [0] & Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[0..2], &[42, 0]);
        // Set supply ElGamal pubkey as [u8;ELGAMAL_PUBKEY_LEN] at offset [2..34]
        write_bytes(&mut instruction_data[2..34], &self.supply_elgamal_pubkey);
        // Set initial decryptable supply as [u8; POD_AE_CIPHERTEXT_LEN] at offset [34..70]
        write_bytes(&mut instruction_data[34..70], &self.decryptable_supply);

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 70) },
        };

        invoke_signed(&instruction, &[self.mint], signers)?;

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
    pub new_supply_elgamal_pubkey: [u8; ELGAMAL_PUBKEY_LEN],
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
        let account_metas = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly(self.instruction_sysvar.key()),
            AccountMeta::readonly_signer(self.confidential_mint_authority.key()),
        ];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [2..34]: new supply ElGamal pubkey (32 bytes, [u8;ELGAMAL_PUBKEY_LEN])
        // -  [34]: proof instruction offset (1 byte, i8)

        let mut instruction_data = [UNINIT_BYTE; 35];

        // Set discriminator as u8 at offset [0] & Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[0..2], &[42, 1]);
        // Set new supply ElGamal pubkey as [u8;ELGAMAL_PUBKEY_LEN] at offset [2..34]
        write_bytes(
            &mut instruction_data[2..34],
            &self.new_supply_elgamal_pubkey,
        );
        // Set proof instruction offset as i8 at offset [34]
        write_bytes(
            &mut instruction_data[34..35],
            &[self.proof_instruction_offset as u8],
        );

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 35) },
        };

        invoke_signed(
            &instruction,
            &[
                self.mint,
                self.instruction_sysvar,
                self.confidential_mint_authority,
            ],
            signers,
        )?;

        Ok(())
    }
}

pub struct UpdateDecryptableSupply<'a> {
    /// The mint to update
    pub mint: &'a AccountInfo,
    /// The confidential mint authority
    pub confidential_mint_authority: &'a AccountInfo,
    /// The new decryptable supply
    pub new_decryptable_supply: [u8; POD_AE_CIPHERTEXT_LEN],
}

impl UpdateDecryptableSupply<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.confidential_mint_authority.key()),
        ];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [2..38]: new decryptable supply (36 bytes, [u8; POD_AE_CIPHERTEXT_LEN])
        let mut instruction_data = [UNINIT_BYTE; 38];
        // Set discriminator as u8 at offset [0] & Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[0..2], &[42, 2]);
        // Set new decryptable supply as [u8; POD_AE_CIPHERTEXT_LEN] at offset [2..38]
        write_bytes(&mut instruction_data[2..38], &self.new_decryptable_supply);

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 38) },
        };

        invoke_signed(
            &instruction,
            &[self.mint, self.confidential_mint_authority],
            signers,
        )?;

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
    pub new_decryptable_supply: [u8; POD_AE_CIPHERTEXT_LEN],
    /// The transfer amount encrypted under the auditor ElGamal public key
    pub mint_amount_auditor_ciphertext_lo: [u8; POD_ELGAMAL_CIPHERTEXT_LEN],
    /// The transfer amount encrypted under the auditor ElGamal public key
    pub mint_amount_auditor_ciphertext_hi: [u8; POD_ELGAMAL_CIPHERTEXT_LEN],
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
        let account_metas = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly(self.instruction_sysvar.key()),
            AccountMeta::readonly(self.verify_ciphertext_commitment_equality.key()),
            AccountMeta::readonly(
                self.verify_batched_grouped_cihertext3_handles_validity
                    .key(),
            ),
            AccountMeta::readonly(self.verify_batched_range_proof_u128.key()),
            AccountMeta::readonly_signer(self.account_owner.key()),
        ];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [2..38]: new decryptable supply (36 bytes, [u8; POD_AE_CIPHERTEXT_LEN])
        // -  [38..70]: mint amount auditor ciphertext lo (32 bytes, [u8; POD_ELGAMAL_CIPHERTEXT_LEN])
        // -  [70..102]: mint amount auditor ciphertext hi (32 bytes, [u8; POD_ELGAMAL_CIPHERTEXT_LEN])
        // -  [102]: equality proof instruction offset (1 byte, i8)
        // -  [103]: ciphertext validity proof instruction offset (1 byte, i8)
        // -  [104]: range proof instruction offset (1 byte, i8)
        let mut instruction_data = [UNINIT_BYTE; 105];

        // Set discriminator as u8 at offset [0] & Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[0..2], &[42, 3]);
        // Set new decryptable supply as [u8; POD_AE_CIPHERTEXT_LEN] at offset [2..38]
        write_bytes(&mut instruction_data[2..38], &self.new_decryptable_supply);
        // Set mint amount auditor ciphertext lo as [u8; POD_ELGAMAL_CIPHERTEXT_LEN] at offset [38..70]
        write_bytes(
            &mut instruction_data[38..70],
            &self.mint_amount_auditor_ciphertext_lo,
        );
        // Set mint amount auditor ciphertext hi as [u8; POD_ELGAMAL_CIPHERTEXT_LEN] at offset [70..102]
        write_bytes(
            &mut instruction_data[70..102],
            &self.mint_amount_auditor_ciphertext_hi,
        );
        // Set equality proof instruction offset as i8 at offset [102]
        write_bytes(
            &mut instruction_data[102..103],
            &[self.equality_proof_instruction_offset as u8],
        );
        // Set ciphertext validity proof instruction offset as i8 at offset [103]
        write_bytes(
            &mut instruction_data[103..104],
            &[self.ciphertext_validity_proof_instruction_offset as u8],
        );
        // Set range proof instruction offset as i8 at offset [104]
        write_bytes(
            &mut instruction_data[104..105],
            &[self.range_proof_instruction_offset as u8],
        );
        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 105) },
        };
        invoke_signed(
            &instruction,
            &[
                self.account,
                self.mint,
                self.instruction_sysvar,
                self.verify_ciphertext_commitment_equality,
                self.verify_batched_grouped_cihertext3_handles_validity,
                self.verify_batched_range_proof_u128,
                self.account_owner,
            ],
            signers,
        )?;

        Ok(())
    }
}

pub struct Burn<'a> {
    /// The SPL Token account to burn from
    pub account: &'a AccountInfo,
    /// The SPL Token mint
    pub mint: &'a AccountInfo,
    /// (Optional) Instructions sysvar if at least one of the `zk_elgamal_proof` instructions is included in the same transaction
    pub instruction_sysvar: &'a AccountInfo,
    /// (Optional) The context state account containing the pre-verified `VerifyCiphertextCommitmentEquality` proof
    pub verify_ciphertext_commitment_equality: &'a AccountInfo,
    /// (Optional) The context state account containing the pre-verified `VerifyBatchedGroupedCiphertext3HandlesValidity` proof
    pub verify_batched_grouped_ciphertext3_handles_validity: &'a AccountInfo,
    /// (Optional) The context state account containing the pre-verified `VerifyBatchedRangeProofU128`
    pub verify_batched_range_proof_u128: &'a AccountInfo,
    /// The single account owner
    pub account_owner: &'a AccountInfo,
    /// The new decryptable balance of the burner if the burn succeeds
    pub new_decryptable_available_balance: [u8; POD_AE_CIPHERTEXT_LEN],
    /// The transfer amount encrypted under the auditor ElGamal public key
    pub burn_amount_auditor_ciphertext_lo: [u8; POD_ELGAMAL_CIPHERTEXT_LEN],
    /// The transfer amount encrypted under the auditor ElGamal public key
    pub burn_amount_auditor_ciphertext_hi: [u8; POD_ELGAMAL_CIPHERTEXT_LEN],
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
        let account_metas = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly(self.instruction_sysvar.key()),
            AccountMeta::readonly(self.verify_ciphertext_commitment_equality.key()),
            AccountMeta::readonly(
                self.verify_batched_grouped_ciphertext3_handles_validity
                    .key(),
            ),
            AccountMeta::readonly(self.verify_batched_range_proof_u128.key()),
            AccountMeta::readonly_signer(self.account_owner.key()),
        ];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [2..38]: new decryptable available balance (36 bytes, [u8; POD_AE_CIPHERTEXT_LEN])
        // -  [38..70]: burn amount auditor ciphertext lo (32 bytes, [u8; POD_ELGAMAL_CIPHERTEXT_LEN])
        // -  [70..102]: burn amount auditor ciphertext hi (32 bytes, [u8; POD_ELGAMAL_CIPHERTEXT_LEN])
        // -  [102]: equality proof instruction offset (1 byte, i8)
        // -  [103]: ciphertext validity proof instruction offset (1 byte, i8)
        // -  [104]: range proof instruction offset (1 byte, i8)
        let mut instruction_data = [UNINIT_BYTE; 105];
        // Set discriminator as u8 at offset [0] & Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[0..2], &[42, 4]);
        // Set new decryptable available balance as [u8; POD_AE_CIPHERTEXT_LEN] at offset [2..38]
        write_bytes(
            &mut instruction_data[2..38],
            &self.new_decryptable_available_balance,
        );
        // Set burn amount auditor ciphertext lo as [u8; POD_ELGAMAL_CIPHERTEXT_LEN] at offset [38..70]
        write_bytes(
            &mut instruction_data[38..70],
            &self.burn_amount_auditor_ciphertext_lo,
        );
        // Set burn amount auditor ciphertext hi as [u8; POD_ELGAMAL_CIPHERTEXT_LEN] at offset [70..102]
        write_bytes(
            &mut instruction_data[70..102],
            &self.burn_amount_auditor_ciphertext_hi,
        );
        // Set equality proof instruction offset as i8 at offset [102]
        write_bytes(
            &mut instruction_data[102..103],
            &[self.equality_proof_instruction_offset as u8],
        );
        // Set ciphertext validity proof instruction offset as i8 at offset [103]
        write_bytes(
            &mut instruction_data[103..104],
            &[self.ciphertext_validity_proof_instruction_offset as u8],
        );
        // Set range proof instruction offset as i8 at offset [104]
        write_bytes(
            &mut instruction_data[104..105],
            &[self.range_proof_instruction_offset as u8],
        );
        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 105) },
        };
        invoke_signed(
            &instruction,
            &[
                self.account,
                self.mint,
                self.instruction_sysvar,
                self.verify_ciphertext_commitment_equality,
                self.verify_batched_grouped_ciphertext3_handles_validity,
                self.verify_batched_range_proof_u128,
                self.account_owner,
            ],
            signers,
        )?;

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
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &[42, 5],
        };

        invoke_signed(&instruction, &[self.mint, self.mint_authority], signers)?;
        Ok(())
    }
}
