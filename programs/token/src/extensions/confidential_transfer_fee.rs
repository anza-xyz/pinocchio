use pinocchio::{
    account_info::AccountInfo,
    cpi::{invoke_signed, slice_invoke_signed},
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

extern crate alloc;
use alloc::vec::Vec;

use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

use super::{
    get_extension_from_bytes, EncryptedBalance, PodElGamalCiphertext, PodElGamalPubkey,
    ELGAMAL_PUBKEY_LEN, POD_AE_CIPHERTEXT_LEN,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ConfidentialTransferFeeConfig {
    /// Optional authority to set the withdraw withheld authority ElGamal key
    pub authority: Pubkey,

    /// Withheld fees from accounts must be encrypted with this ElGamal key.
    ///
    /// Note that whoever holds the ElGamal private key for this ElGamal public
    /// key has the ability to decode any withheld fee amount that are
    /// associated with accounts. When combined with the fee parameters, the
    /// withheld fee amounts can reveal information about transfer amounts.
    pub withdraw_withheld_authority_elgamal_pubkey: PodElGamalPubkey,

    /// If `false`, the harvest of withheld tokens to mint is rejected.
    pub harvest_to_mint_enabled: u8,

    /// Withheld confidential transfer fee tokens that have been moved to the
    /// mint for withdrawal.
    pub withheld_amount: PodElGamalCiphertext,
}

impl super::Extension for ConfidentialTransferFeeConfig {
    const TYPE: super::ExtensionType = super::ExtensionType::ConfidentialTransferFeeConfig;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl ConfidentialTransferFeeConfig {
    /// The length of the `ConfidentialTransferFeeConfig` account data.
    pub const LEN: usize = core::mem::size_of::<ConfidentialTransferFeeConfig>();

    /// Return a `ConfidentialTransferFeeConfig` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &pinocchio::account_info::AccountInfo,
    ) -> Result<&ConfidentialTransferFeeConfig, pinocchio::program_error::ProgramError> {
        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(pinocchio::program_error::ProgramError::InvalidAccountData)
    }
}

/// Confidential transfer fee
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ConfidentialTransferFeeAmount {
    /// Amount withheld during confidential transfers, to be harvest to the mint
    pub withheld_amount: EncryptedBalance,
}

impl super::Extension for ConfidentialTransferFeeAmount {
    const TYPE: super::ExtensionType = super::ExtensionType::ConfidentialTransferFeeAmount;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::TokenAccount;
}

impl ConfidentialTransferFeeAmount {
    /// The length of the `ConfidentialTransferFeeAmount` account data.
    pub const LEN: usize = core::mem::size_of::<ConfidentialTransferFeeAmount>();

    /// Return a `ConfidentialTransferFeeAmount` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &pinocchio::account_info::AccountInfo,
    ) -> Result<&ConfidentialTransferFeeAmount, pinocchio::program_error::ProgramError> {
        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(pinocchio::program_error::ProgramError::InvalidAccountData)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WithheldTokensInfo {
    /// The available balance
    pub withheld_amount: PodElGamalCiphertext,
}

// Instructions

pub struct InitializeConfidentialTransferFeeConfig<'a> {
    /// The mint to initialize the confidential transfer fee config
    pub mint: &'a AccountInfo,
    /// The authority to set the withdraw withheld authority ElGamal key
    pub authority: Option<Pubkey>,
    /// The ElGamal public key for the withdraw withheld authority
    pub withdraw_withheld_authority_elgamal_pubkey: [u8; ELGAMAL_PUBKEY_LEN],
}

impl InitializeConfidentialTransferFeeConfig<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.authority.as_ref().unwrap()),
        ];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [2..38]: authority (32 bytes, Pubkey)
        // -  [38..50]: withdraw withheld authority ElGamal public key (32 bytes, ElGamalPubkey)

        let mut instruction_data = [UNINIT_BYTE; 70];

        // Set the instruction discriminator
        write_bytes(&mut instruction_data[0..2], &[37, 0]);

        // Set the authority
        if let Some(authority) = self.authority {
            write_bytes(&mut instruction_data[2..38], &authority);
        } else {
            write_bytes(&mut instruction_data[2..38], &Pubkey::default());
        }

        // Set the withdraw withheld authority ElGamal public key
        write_bytes(
            &mut instruction_data[38..70],
            &self.withdraw_withheld_authority_elgamal_pubkey,
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 70) },
        };

        invoke_signed(&instruction, &[self.mint], signers)?;

        Ok(())
    }
}

pub struct WithdrawWithheldTokensFromMint<'a> {
    /// The token mint. Must include the `TransferFeeConfig`
    pub mint: &'a AccountInfo,
    /// The fee receiver account. Must include the
    ///      `TransferFeeAmount` and `ConfidentialTransferAccount` extensions.
    pub fee_receiver: &'a AccountInfo,
    /// sysvar account
    pub sysvar: &'a AccountInfo,
    // record account
    pub record: &'a AccountInfo,
    /// Relative location of the `ProofInstruction::VerifyWithdrawWithheld`
    /// instruction to the `WithdrawWithheldTokensFromMint` instruction in
    /// the transaction. If the offset is `0`, then use a context state
    /// account for the proof.
    pub proof_instruction_offset: i8,
    /// The new decryptable balance in the destination token account.
    pub new_decryptable_available_balance: [u8; POD_AE_CIPHERTEXT_LEN],
}

impl WithdrawWithheldTokensFromMint<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::writable(self.fee_receiver.key()),
            AccountMeta::readonly(self.sysvar.key()),
            AccountMeta::writable(self.record.key()),
        ];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [2]: proof instruction offset (1 byte, i8)
        // -  [3...39]: new decryptable available balance (36 bytes, [u8; POD_AE_CIPHERTEXT_LEN])

        let mut instruction_data = [UNINIT_BYTE; 39];

        // Set the instruction discriminator
        write_bytes(
            &mut instruction_data[0..3],
            &[37, 1, self.proof_instruction_offset as u8],
        );
        // Set the new decryptable available balance
        write_bytes(
            &mut instruction_data[3..39],
            &self.new_decryptable_available_balance,
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 39) },
        };

        invoke_signed(
            &instruction,
            &[self.mint, self.fee_receiver, self.sysvar, self.record],
            signers,
        )?;

        Ok(())
    }
}

pub struct WithdrawWithheldTokensFromAccounts<'a> {
    /// The token mint. Must include the `TransferFeeConfig`
    pub mint: &'a AccountInfo,
    /// The fee receiver account. Must include the
    ///      `TransferFeeAmount` and `ConfidentialTransferAccount` extensions.
    pub fee_receiver: &'a AccountInfo,
    /// sysvar account
    pub sysvar: &'a AccountInfo,
    // record account
    pub record: &'a AccountInfo,
    /// Relative location of the `ProofInstruction::VerifyWithdrawWithheld`
    /// instruction to the `WithdrawWithheldTokensFromMint` instruction in
    /// the transaction. If the offset is `0`, then use a context state
    /// account for the proof.
    pub proof_instruction_offset: i8,
    /// Source accounts to withdraw from
    pub source_accounts: Vec<&'a AccountInfo>,
}

impl WithdrawWithheldTokensFromAccounts<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let mut account_metas = Vec::with_capacity(4 + self.source_accounts.len());

        account_metas.extend_from_slice(&[
            AccountMeta::writable(self.mint.key()),
            AccountMeta::writable(self.fee_receiver.key()),
            AccountMeta::readonly(self.sysvar.key()),
            AccountMeta::writable(self.record.key()),
        ]);

        for source_account in &self.source_accounts {
            account_metas.push(AccountMeta::readonly(source_account.key()));
        }

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [2]: proof instruction offset (1 byte, i8)

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[37, 2, self.proof_instruction_offset as u8],
        };

        let mut account_infos = Vec::with_capacity(4 + self.source_accounts.len());

        account_infos.extend_from_slice(&[self.mint, self.fee_receiver, self.sysvar, self.record]);

        for source_account in &self.source_accounts {
            account_infos.push(source_account);
        }

        slice_invoke_signed(&instruction, account_infos.as_slice(), signers)?;

        Ok(())
    }
}

pub struct HarvestWithheldTokensToMint<'a, const SOURCE_ACCOUNTS: usize> {
    /// The mint to enable harvest to mint
    pub mint: &'a AccountInfo,
    /// Source accounts to harvest from
    pub source_accounts: [&'a AccountInfo; SOURCE_ACCOUNTS],
}

impl HarvestWithheldTokensToMint<'_, 1> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly(self.source_accounts[0].key()),
        ];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[37, 3],
        };

        invoke_signed(&instruction, &[self.mint], signers)?;

        Ok(())
    }
}

pub struct EnableHarvestToMint<'a> {
    /// The mint to enable harvest to mint
    pub mint: &'a AccountInfo,
    /// The confidential transfer fee authority.
    pub authority: &'a AccountInfo,
}

impl EnableHarvestToMint<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[37, 4],
        };

        invoke_signed(&instruction, &[self.mint, self.authority], signers)?;

        Ok(())
    }
}

pub struct DisableHarvestToMint<'a> {
    /// The mint to disable harvest to mint
    pub mint: &'a AccountInfo,
    /// The confidential transfer fee authority.
    pub authority: &'a AccountInfo,
}

impl DisableHarvestToMint<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[37, 5],
        };

        invoke_signed(&instruction, &[self.mint, self.authority], signers)?;

        Ok(())
    }
}
