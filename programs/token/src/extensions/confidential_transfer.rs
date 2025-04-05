use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    system_program, ProgramError, ProgramResult,
};

use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

use super::ElagamalPubkey;

// Define necessary types locally to avoid external dependencies
pub const POD_AE_CIPHERTEXT_LEN: usize = 36;

/// Local definition mirroring spl_token_confidential_transfer::pod::PodElGamalCiphertext
pub const POD_ELGAMAL_CIPHERTEXT_LEN: usize = 64;

/// Local definition mirroring spl_token_confidential_transfer::pod::PodElGamalCiphertext
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C)]
pub struct PodElGamalCiphertext(pub [u8; POD_ELGAMAL_CIPHERTEXT_LEN]);

/// Local definition mirroring spl_token_confidential_transfer::pod::PodAeCiphertext
#[derive(Clone, Copy, Debug, PartialEq, Default)] // Add Default for convenience
#[repr(C)]
pub struct PodAeCiphertext(pub [u8; POD_AE_CIPHERTEXT_LEN]);

/// Alias for clarity, mirroring spl_token_confidential_transfer::instruction::DecryptableBalance
pub type DecryptableBalance = PodAeCiphertext;

// Instructions

/// Initialize a new mint for a confidential transfer.
pub struct InitializeMint<'a> {
    pub mint: &'a AccountInfo,
    /// Authority to modify the `ConfidentialTransferMint` configuration and to
    /// approve new accounts.
    pub authority: Option<&'a Pubkey>,
    /// Determines if newly configured accounts must be approved by the
    /// `authority` before they may be used by the user.
    pub auto_approve_new_accounts: bool,
    /// New authority to decode any transfer amount in a confidential transfer.
    pub auditor_elgamal_pubkey: Option<&'a ElagamalPubkey>,
}

impl InitializeMint<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [2]: auto_approve_new_accounts (1 byte, u8)
        // -  [3..35]: authority (32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 35];
        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[27]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[0]);
        // Set auto_approve_new_accounts as u8 at offset [2]
        write_bytes(
            &mut instruction_data[2..3],
            &[self.auto_approve_new_accounts as u8],
        );

        if let Some(authority) = self.authority {
            write_bytes(&mut instruction_data[3..35], authority);
        } else {
            write_bytes(&mut instruction_data[3..35], &Pubkey::default());
        }

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 35) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}

pub struct UpdateMint<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// `ConfidentialTransfer` transfer mint authority..
    pub mint_authority: &'a Pubkey,
    /// Determines if newly configured accounts must be approved by the
    /// `authority` before they may be used by the user.
    pub auto_approve_new_accounts: bool,
    /// New authority to decode any transfer amount in a confidential transfer.
    pub auditor_elgamal_pubkey: Option<&'a ElagamalPubkey>,
}

impl UpdateMint<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [1..33]: mint_authority (32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 34];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[27]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[1]);
        // Set mint_authority as Pubkey at offset [1..33]
        write_bytes(&mut instruction_data[2..34], self.mint_authority);

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 34) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}

// Add ConfigureAccount before ConfigureAccountWithRegistry
pub struct ConfigureAccount<'a> {
    /// Token account to configure.
    pub token_account: &'a AccountInfo,
    /// Mint associated with the token account.
    pub mint: &'a AccountInfo,
    /// Token account owner.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
    /// The ElGamal public key for the account.
    pub elgamal_pk: &'a ElagamalPubkey,
    /// The decryptable balance (typically ciphertext corresponding to 0)
    /// encrypted with the `elgamal_pk`.
    pub decryptable_zero_balance: &'a DecryptableBalance,
    /// Optional payer account for reallocation if the token account is too small.
    pub payer: Option<&'a AccountInfo>,
}

impl ConfigureAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Base accounts
        let mut account_metas = vec![
            AccountMeta::writable(self.token_account.key()),
            AccountMeta::readonly(self.mint.key()),
        ];
        let mut accounts = vec![self.token_account, self.mint];

        // Add optional payer and implicit system program if payer is provided
        if let Some(payer_info) = self.payer {
            account_metas.push(AccountMeta::writable_signer(payer_info.key()));
            // Implicitly add system program metadata
            account_metas.push(AccountMeta::readonly(&system_program::ID));
            accounts.push(payer_info);
        }

        // Add authority and potential multisig signers
        account_metas.push(AccountMeta::readonly_signer(self.authority.key())); // Start assuming authority is signer
        accounts.push(self.authority);

        let authority_meta_index = account_metas.len() - 1;
        for multisig_signer in self.multisig_signers.iter() {
            // If multisig signers are present, authority is not a direct signer
            account_metas[authority_meta_index] = AccountMeta::readonly(self.authority.key());
            account_metas.push(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts.push(multisig_signer);
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 2 (ConfigureAccount)
        // -  [2..34]: elgamal_pk (32 bytes, ElgamalPubkey)
        // -  [34..70]: decryptable_zero_balance (36 bytes, PodAeCiphertext/DecryptableBalance)
        const DATA_LEN: usize = 1 + 1 + super::ELGAMAL_PUBKEY_LEN + POD_AE_CIPHERTEXT_LEN; // 1 + 1 + 32 + 36 = 70
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];

        write_bytes(&mut instruction_data[0..1], &[27]); // Main discriminator
        write_bytes(&mut instruction_data[1..2], &[2]); // ConfigureAccount discriminator
        write_bytes(
            &mut instruction_data[2..(2 + super::ELGAMAL_PUBKEY_LEN)],
            &self.elgamal_pk.0,
        ); // ElGamal PK bytes
        write_bytes(
            &mut instruction_data[(2 + super::ELGAMAL_PUBKEY_LEN)..DATA_LEN],
            &self.decryptable_zero_balance.0,
        ); // Decryptable zero balance ciphertext bytes

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, DATA_LEN) },
        };

        // Pass account slices to invoke_signed
        invoke_signed(&instruction, &accounts, signers)
    }
}

pub struct ConfigureAccountWithRegistry<'a> {
    /// Token account to configure.
    pub token_account: &'a AccountInfo,
    /// Mint associated with the token account.
    pub mint: &'a AccountInfo,
    /// ElGamal registry account containing the ElGamal public key.
    pub elgamal_registry_account: &'a AccountInfo,
    /// Optional payer account for reallocation if the token account is too small.
    pub payer: Option<&'a AccountInfo>,
}

impl ConfigureAccountWithRegistry<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let mut account_metas = vec![
            AccountMeta::writable(self.token_account.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly(self.elgamal_registry_account.key()),
        ];

        // Accounts to pass to invoke_signed
        let mut accounts = vec![self.token_account, self.mint, self.elgamal_registry_account];

        // Add optional payer and implicit system program if payer is provided
        if let Some(payer_info) = self.payer {
            account_metas.push(AccountMeta::writable_signer(payer_info.key()));
            // Implicitly add system program metadata
            account_metas.push(AccountMeta::readonly(&system_program::ID));
            accounts.push(payer_info);
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 14 (ConfigureAccountWithRegistry)
        let mut instruction_data = [UNINIT_BYTE; 2];
        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[27]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[14]);

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 2) },
        };

        // Pass account slices to invoke_signed
        invoke_signed(&instruction, &accounts, signers)
    }
}

pub struct ApproveAccount<'a> {
    /// The SPL Token account to approve.
    pub token_account: &'a AccountInfo,
    /// The SPL Token mint.
    pub mint: &'a AccountInfo,
    /// Confidential transfer mint authority.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
}

impl ApproveAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let mut account_metas = vec![
            AccountMeta::writable(self.token_account.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly_signer(self.authority.key()), // Assume authority is always a signer here
        ];

        // Accounts to pass to invoke_signed
        let mut accounts = vec![self.token_account, self.mint, self.authority];

        // Add multisig signers if provided
        for multisig_signer in self.multisig_signers.iter() {
            // If multisig signers are present, the authority itself is not a direct signer
            if let Some(authority_meta) = account_metas.get_mut(2) {
                *authority_meta = AccountMeta::readonly(self.authority.key());
            }
            account_metas.push(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts.push(multisig_signer);
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 3 (ApproveAccount)
        let mut instruction_data = [UNINIT_BYTE; 2];
        write_bytes(&mut instruction_data[0..1], &[27]);
        write_bytes(&mut instruction_data[1..2], &[3]);

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 2) },
        };

        invoke_signed(&instruction, &accounts, signers)
    }
}

/// Creates the CPI instruction for `EmptyAccount` based on the underlying
/// `inner_empty_account` logic from the Token-2022 program.
///
/// Note: This wrapper creates *only* the `EmptyAccount` instruction.
/// The caller is responsible for managing the associated ZK proof instruction
/// (`VerifyZeroCiphertext`) or context state account required by the Token-2022
/// program, ensuring it's correctly placed relative to this instruction
/// or provided via the `proof_account` field.
pub struct EmptyAccount<'a> {
    /// The SPL Token account to empty.
    pub token_account: &'a AccountInfo,
    /// The account owner or delegate.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
    /// Proof account: Instructions sysvar or context state account.
    pub proof_account: &'a AccountInfo,
    /// Optional record account if proof data is stored there.
    pub record_account: Option<&'a AccountInfo>,
    /// Relative offset of the proof instruction, or 0 if using context state account.
    pub proof_instruction_offset: i8,
}

impl EmptyAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Base accounts
        let mut account_metas = vec![
            AccountMeta::writable(self.token_account.key()),
            // Proof account (Sysvar or Context State)
            AccountMeta::readonly(self.proof_account.key()),
        ];
        let mut accounts = vec![self.token_account, self.proof_account];

        // Add optional record account if offset is non-zero and account is provided
        if self.proof_instruction_offset != 0 {
            if let Some(record_acc) = self.record_account {
                account_metas.push(AccountMeta::readonly(record_acc.key()));
                accounts.push(record_acc);
            }
            // Note: The original instruction differentiates between ProofData::InstructionData
            // and ProofData::RecordAccount within the InstructionOffset case.
            // Our wrapper doesn't have visibility into the ProofData enum, so we rely
            // on the caller providing record_account only when appropriate.
            // If proof_instruction_offset is non-zero but record_account is None,
            // we assume the proof data is in the sysvar itself.
        }

        // Add authority and potential multisig signers
        account_metas.push(AccountMeta::readonly_signer(self.authority.key())); // Start assuming authority is signer
        accounts.push(self.authority);

        let authority_meta_index = account_metas.len() - 1;
        for multisig_signer in self.multisig_signers.iter() {
            // If multisig signers are present, authority is not a direct signer
            account_metas[authority_meta_index] = AccountMeta::readonly(self.authority.key());
            account_metas.push(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts.push(multisig_signer);
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 4 (EmptyAccount)
        // -  [2]: proof_instruction_offset (1 byte, i8)
        let mut instruction_data = [UNINIT_BYTE; 3];
        write_bytes(&mut instruction_data[0..1], &[27]);
        write_bytes(&mut instruction_data[1..2], &[4]);
        write_bytes(
            &mut instruction_data[2..3],
            &[self.proof_instruction_offset as u8],
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 3) },
        };

        invoke_signed(&instruction, &accounts, signers)
    }
}

pub struct Deposit<'a> {
    /// The destination SPL Token account (must have ConfidentialTransfer extension).
    pub token_account: &'a AccountInfo,
    /// The SPL Token mint.
    pub mint: &'a AccountInfo,
    /// The owner or delegate of the source non-confidential token account.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
    /// Amount of tokens to deposit.
    pub amount: u64,
    /// Expected number of decimals for the mint.
    pub decimals: u8,
}

impl Deposit<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let mut account_metas = vec![
            AccountMeta::writable(self.token_account.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly_signer(self.authority.key()), // Start assuming authority is signer
        ];
        let mut accounts = vec![self.token_account, self.mint, self.authority];

        // Handle multisig signers
        let authority_meta_index = account_metas.len() - 1;
        for multisig_signer in self.multisig_signers.iter() {
            // If multisig signers are present, authority is not a direct signer
            account_metas[authority_meta_index] = AccountMeta::readonly(self.authority.key());
            account_metas.push(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts.push(multisig_signer);
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 5 (Deposit)
        // -  [2..10]: amount (8 bytes, u64)
        // -  [10]: decimals (1 byte, u8)
        let mut instruction_data = [UNINIT_BYTE; 11];
        write_bytes(&mut instruction_data[0..1], &[27]); // Main discriminator
        write_bytes(&mut instruction_data[1..2], &[5]); // Deposit discriminator
        write_bytes(&mut instruction_data[2..10], &self.amount.to_le_bytes()); // Amount
        write_bytes(&mut instruction_data[10..11], &[self.decimals]); // Decimals

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 11) },
        };

        invoke_signed(&instruction, &accounts, signers)
    }
}

/// Creates the CPI instruction for `Withdraw` based on the underlying
/// `inner_withdraw` logic from the Token-2022 program.
///
/// Note: This wrapper creates *only* the `Withdraw` instruction.
/// The caller is responsible for managing the associated ZK proof instructions
/// (`VerifyCiphertextCommitmentEquality`, `VerifyBatchedRangeProofU64`)
/// or context state accounts required by the Token-2022 program, ensuring
/// they are correctly placed relative to this instruction or provided via
/// the proof account fields.
pub struct Withdraw<'a> {
    /// The source SPL Token account (must have ConfidentialTransfer extension).
    pub token_account: &'a AccountInfo,
    /// The SPL Token mint.
    pub mint: &'a AccountInfo,
    /// The account owner or delegate.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
    /// Instructions sysvar (optional, required if either proof offset is non-zero).
    pub sysvar_instructions_account: Option<&'a AccountInfo>,
    /// Equality proof account (optional, context state or record account).
    pub equality_proof_account: Option<&'a AccountInfo>,
    /// Range proof account (optional, context state or record account).
    pub range_proof_account: Option<&'a AccountInfo>,
    /// Amount of tokens to withdraw.
    pub amount: u64,
    /// Expected number of decimals for the mint.
    pub decimals: u8,
    /// The new decryptable balance ciphertext after the withdrawal succeeds.
    pub new_decryptable_available_balance: PodAeCiphertext,
    /// Relative offset of the equality proof instruction, or 0 if using context state account.
    pub equality_proof_instruction_offset: i8,
    /// Relative offset of the range proof instruction, or 0 if using context state account.
    pub range_proof_instruction_offset: i8,
}

impl Withdraw<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Base accounts
        let mut account_metas = vec![
            AccountMeta::writable(self.token_account.key()),
            AccountMeta::readonly(self.mint.key()),
        ];
        let mut accounts = vec![self.token_account, self.mint];

        // Add optional Instructions sysvar if needed
        if self.equality_proof_instruction_offset != 0 || self.range_proof_instruction_offset != 0 {
            if let Some(sysvar_acc) = self.sysvar_instructions_account {
                account_metas.push(AccountMeta::readonly(sysvar_acc.key()));
                accounts.push(sysvar_acc);
            } else {
                // If proofs are inline, sysvar MUST be provided
                return Err(ProgramError::InvalidArgument); // Or a more specific error
            }
        }

        // Add optional proof accounts (Context State or Record)
        // Note: We rely on the caller providing the correct account type based on the offset
        if let Some(equality_proof_acc) = self.equality_proof_account {
            account_metas.push(AccountMeta::readonly(equality_proof_acc.key()));
            accounts.push(equality_proof_acc);
        }
        if let Some(range_proof_acc) = self.range_proof_account {
            account_metas.push(AccountMeta::readonly(range_proof_acc.key()));
            accounts.push(range_proof_acc);
        }

        // Add authority and potential multisig signers
        account_metas.push(AccountMeta::readonly_signer(self.authority.key())); // Start assuming authority is signer
        accounts.push(self.authority);

        let authority_meta_index = account_metas.len() - 1;
        for multisig_signer in self.multisig_signers.iter() {
            // If multisig signers are present, authority is not a direct signer
            account_metas[authority_meta_index] = AccountMeta::readonly(self.authority.key());
            account_metas.push(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts.push(multisig_signer);
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 6 (Withdraw)
        // -  [2..10]: amount (8 bytes, u64)
        // -  [10]: decimals (1 byte, u8)
        // -  [11..47]: new_decryptable_available_balance (36 bytes, PodAeCiphertext)
        // -  [47]: equality_proof_instruction_offset (1 byte, i8)
        // -  [48]: range_proof_instruction_offset (1 byte, i8)
        const DATA_LEN: usize = 1 + 1 + 8 + 1 + POD_AE_CIPHERTEXT_LEN + 1 + 1;
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];

        write_bytes(&mut instruction_data[0..1], &[27]); // Main discriminator
        write_bytes(&mut instruction_data[1..2], &[6]); // Withdraw discriminator
        write_bytes(&mut instruction_data[2..10], &self.amount.to_le_bytes()); // Amount
        write_bytes(&mut instruction_data[10..11], &[self.decimals]); // Decimals
        write_bytes(
            &mut instruction_data[11..(11 + POD_AE_CIPHERTEXT_LEN)],
            &self.new_decryptable_available_balance.0,
        ); // Balance ciphertext bytes
        write_bytes(
            &mut instruction_data[DATA_LEN - 2..DATA_LEN - 1],
            &[self.equality_proof_instruction_offset as u8],
        ); // Equality offset
        write_bytes(
            &mut instruction_data[DATA_LEN - 1..DATA_LEN],
            &[self.range_proof_instruction_offset as u8],
        ); // Range offset

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, DATA_LEN) },
        };

        invoke_signed(&instruction, &accounts, signers)
    }
}

pub struct ApplyPendingBalance<'a> {
    /// The SPL Token account holding the pending balance.
    pub token_account: &'a AccountInfo,
    /// The account owner.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
    /// The expected number of pending balance credits to apply.
    pub expected_pending_balance_credit_counter: u64,
    /// The new decryptable balance ciphertext after applying the pending balance.
    pub new_decryptable_available_balance: PodAeCiphertext,
}

impl ApplyPendingBalance<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let mut account_metas = vec![
            AccountMeta::writable(self.token_account.key()),
            AccountMeta::readonly_signer(self.authority.key()), // Start assuming authority is signer
        ];
        let mut accounts = vec![self.token_account, self.authority];

        // Handle multisig signers
        let authority_meta_index = account_metas.len() - 1;
        for multisig_signer in self.multisig_signers.iter() {
            // If multisig signers are present, authority is not a direct signer
            account_metas[authority_meta_index] = AccountMeta::readonly(self.authority.key());
            account_metas.push(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts.push(multisig_signer);
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 8 (ApplyPendingBalance)
        // -  [2..10]: expected_pending_balance_credit_counter (8 bytes, u64)
        // -  [10..46]: new_decryptable_available_balance (36 bytes, PodAeCiphertext)
        const DATA_LEN: usize = 1 + 1 + 8 + POD_AE_CIPHERTEXT_LEN;
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];

        write_bytes(&mut instruction_data[0..1], &[27]); // Main discriminator
        write_bytes(&mut instruction_data[1..2], &[8]); // ApplyPendingBalance discriminator
        write_bytes(
            &mut instruction_data[2..10],
            &self.expected_pending_balance_credit_counter.to_le_bytes(),
        ); // Counter
        write_bytes(
            &mut instruction_data[10..DATA_LEN],
            &self.new_decryptable_available_balance.0,
        ); // Balance ciphertext bytes

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, DATA_LEN) },
        };

        invoke_signed(&instruction, &accounts, signers)
    }
}

pub struct AllowConfidentialCredits<'a> {
    /// The SPL Token account to allow confidential credits for.
    pub token_account: &'a AccountInfo,
    /// The account owner or delegate.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
}

impl AllowConfidentialCredits<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let mut account_metas = vec![
            AccountMeta::writable(self.token_account.key()),
            AccountMeta::readonly_signer(self.authority.key()), // Start assuming authority is signer
        ];
        let mut accounts = vec![self.token_account, self.authority];

        // Handle multisig signers
        let authority_meta_index = account_metas.len() - 1; // Index is 1
        for multisig_signer in self.multisig_signers.iter() {
            // If multisig signers are present, authority is not a direct signer
            account_metas[authority_meta_index] = AccountMeta::readonly(self.authority.key());
            account_metas.push(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts.push(multisig_signer);
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 9 (AllowConfidentialCredits)
        const DATA_LEN: usize = 2;
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];

        write_bytes(&mut instruction_data[0..1], &[27]); // Main discriminator
        write_bytes(&mut instruction_data[1..2], &[9]); // AllowConfidentialCredits discriminator

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, DATA_LEN) },
        };

        invoke_signed(&instruction, &accounts, signers)
    }
}

pub struct AllowNonConfidentialCredits<'a> {
    /// The SPL Token account to allow non-confidential credits for.
    pub token_account: &'a AccountInfo,
    /// The account owner or delegate.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
}

impl AllowNonConfidentialCredits<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let mut account_metas = vec![
            AccountMeta::writable(self.token_account.key()),
            AccountMeta::readonly_signer(self.authority.key()), // Start assuming authority is signer
        ];
        let mut accounts = vec![self.token_account, self.authority];

        // Handle multisig signers
        let authority_meta_index = account_metas.len() - 1; // Index is 1
        for multisig_signer in self.multisig_signers.iter() {
            // If multisig signers are present, authority is not a direct signer
            account_metas[authority_meta_index] = AccountMeta::readonly(self.authority.key());
            account_metas.push(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts.push(multisig_signer);
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 10 (AllowNonConfidentialCredits)
        const DATA_LEN: usize = 2;
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];

        write_bytes(&mut instruction_data[0..1], &[27]); // Main discriminator
        write_bytes(&mut instruction_data[1..2], &[10]); // AllowNonConfidentialCredits discriminator

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, DATA_LEN) },
        };

        invoke_signed(&instruction, &accounts, signers)
    }
}

pub struct DisableConfidentialCredits<'a> {
    /// The SPL Token account to disable confidential credits for.
    pub token_account: &'a AccountInfo,
    /// The account owner or delegate.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
}

impl DisableConfidentialCredits<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let mut account_metas = vec![
            AccountMeta::writable(self.token_account.key()),
            AccountMeta::readonly_signer(self.authority.key()), // Start assuming authority is signer
        ];
        let mut accounts = vec![self.token_account, self.authority];

        // Handle multisig signers
        let authority_meta_index = account_metas.len() - 1; // Index is 1
        for multisig_signer in self.multisig_signers.iter() {
            // If multisig signers are present, authority is not a direct signer
            account_metas[authority_meta_index] = AccountMeta::readonly(self.authority.key());
            account_metas.push(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts.push(multisig_signer);
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 11 (DisableConfidentialCredits)
        const DATA_LEN: usize = 2;
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];

        write_bytes(&mut instruction_data[0..1], &[27]); // Main discriminator
        write_bytes(&mut instruction_data[1..2], &[11]); // DisableConfidentialCredits discriminator

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, DATA_LEN) },
        };

        invoke_signed(&instruction, &accounts, signers)
    }
}

pub struct DisableNonConfidentialCredits<'a> {
    /// The SPL Token account to disable non-confidential credits for.
    pub token_account: &'a AccountInfo,
    /// The account owner or delegate.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
}

impl DisableNonConfidentialCredits<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let mut account_metas = vec![
            AccountMeta::writable(self.token_account.key()),
            AccountMeta::readonly_signer(self.authority.key()), // Start assuming authority is signer
        ];
        let mut accounts = vec![self.token_account, self.authority];

        // Handle multisig signers
        let authority_meta_index = account_metas.len() - 1; // Index is 1
        for multisig_signer in self.multisig_signers.iter() {
            // If multisig signers are present, authority is not a direct signer
            account_metas[authority_meta_index] = AccountMeta::readonly(self.authority.key());
            account_metas.push(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts.push(multisig_signer);
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 12 (DisableNonConfidentialCredits)
        const DATA_LEN: usize = 2;
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];

        write_bytes(&mut instruction_data[0..1], &[27]); // Main discriminator
        write_bytes(&mut instruction_data[1..2], &[12]); // DisableNonConfidentialCredits discriminator

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, DATA_LEN) },
        };

        invoke_signed(&instruction, &accounts, signers)
    }
}

/// Creates the CPI instruction for `EnableConfidentialCredits` based on the underlying
/// logic from the Token-2022 program.
///
/// Note: This wrapper creates *only* the `EnableConfidentialCredits` instruction.
/// The caller is responsible for managing the associated ZK proof instruction
/// (`VerifyZeroCiphertext`) or context state account required by the Token-2022
/// program, ensuring it's correctly placed relative to this instruction
/// or provided via the `proof_account` field.
pub struct EnableConfidentialCredits<'a> {
    /// The SPL Token account to enable confidential credits for.
    pub token_account: &'a AccountInfo,
    /// The account owner or delegate.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
    /// Proof account: Instructions sysvar or context state account or record account.
    pub proof_account: &'a AccountInfo,
    /// Optional record account if proof data is stored there and referenced by offset.
    pub record_account: Option<&'a AccountInfo>,
    /// Relative offset of the proof instruction, or 0 if using context state/record account directly.
    pub proof_instruction_offset: i8,
}

impl EnableConfidentialCredits<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Base accounts needed regardless of proof location type
        let mut account_metas = vec![
            AccountMeta::writable(self.token_account.key()),
            // Proof account (Sysvar, Context State, or Record Account directly)
            AccountMeta::readonly(self.proof_account.key()),
        ];
        let mut accounts = vec![self.token_account, self.proof_account];

        // Add optional record account only if proof is via instruction offset
        // and a record account is explicitly provided for that purpose.
        if self.proof_instruction_offset != 0 {
            if let Some(record_acc) = self.record_account {
                account_metas.push(AccountMeta::readonly(record_acc.key()));
                accounts.push(record_acc);
            }
            // If offset is non-zero but record_account is None, proof is in sysvar (proof_account).
        }
        // If offset is zero, proof is directly in proof_account (Context or Record).

        // Add authority and potential multisig signers
        account_metas.push(AccountMeta::readonly_signer(self.authority.key())); // Start assuming authority is signer
        accounts.push(self.authority);

        let authority_meta_index = account_metas.len() - 1;
        for multisig_signer in self.multisig_signers.iter() {
            // If multisig signers are present, authority is not a direct signer
            account_metas[authority_meta_index] = AccountMeta::readonly(self.authority.key());
            account_metas.push(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts.push(multisig_signer);
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 13 (EnableConfidentialCredits)
        // -  [2]: proof_instruction_offset (1 byte, i8)
        const DATA_LEN: usize = 3;
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];
        write_bytes(&mut instruction_data[0..1], &[27]); // Main discriminator
        write_bytes(&mut instruction_data[1..2], &[13]); // EnableConfidentialCredits discriminator
        write_bytes(
            &mut instruction_data[2..3],
            &[self.proof_instruction_offset as u8],
        ); // Proof offset

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, DATA_LEN) },
        };

        invoke_signed(&instruction, &accounts, signers)
    }
}

pub struct EnableNonConfidentialCredits<'a> {
    /// The SPL Token account to enable non-confidential credits for.
    pub token_account: &'a AccountInfo,
    /// The account owner or delegate.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
}

impl EnableNonConfidentialCredits<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let mut account_metas = vec![
            AccountMeta::writable(self.token_account.key()),
            AccountMeta::readonly_signer(self.authority.key()), // Start assuming authority is signer
        ];
        let mut accounts = vec![self.token_account, self.authority];

        // Handle multisig signers
        let authority_meta_index = account_metas.len() - 1; // Index is 1
        for multisig_signer in self.multisig_signers.iter() {
            // If multisig signers are present, authority is not a direct signer
            account_metas[authority_meta_index] = AccountMeta::readonly(self.authority.key());
            account_metas.push(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts.push(multisig_signer);
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 15 (EnableNonConfidentialCredits)
        const DATA_LEN: usize = 2;
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];

        write_bytes(&mut instruction_data[0..1], &[27]); // Main discriminator
        write_bytes(&mut instruction_data[1..2], &[15]); // EnableNonConfidentialCredits discriminator

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, DATA_LEN) },
        };

        invoke_signed(&instruction, &accounts, signers)
    }
}

/// Creates the CPI instruction for the standard `Transfer` (non-fee) based on
/// the underlying confidential transfer logic from the Token-2022 program.
///
/// Note: This wrapper corresponds to `ConfidentialTransferInstruction::Transfer`.
/// For transfers involving confidential fees, use the `TransferWithFee` wrapper.
///
/// The caller is responsible for managing the associated ZK proof instructions
/// (`VerifyCiphertextCommitmentEquality`, `VerifyTransferAmountCiphertextValidity`,
/// `VerifyBatchedRangeProofU128`) or context state accounts required by the
/// Token-2022 program, ensuring they are correctly placed relative to this
/// instruction or provided via the appropriate proof account fields.
pub struct Transfer<'a> {
    /// The source SPL Token account (must have ConfidentialTransfer extension).
    pub source_token_account: &'a AccountInfo,
    /// The destination SPL Token account (must have ConfidentialTransfer extension).
    pub destination_token_account: &'a AccountInfo,
    /// The SPL Token mint.
    pub mint: &'a AccountInfo,
    /// The source account owner or delegate.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
    /// Instructions sysvar (optional, required if any proof offset is non-zero).
    pub sysvar_instructions_account: Option<&'a AccountInfo>,
    /// Equality proof account (optional, context state or record account).
    pub equality_proof_account: Option<&'a AccountInfo>,
    /// Transfer amount ciphertext validity proof account (optional, context state or record account).
    pub transfer_amount_ciphertext_validity_proof_account: Option<&'a AccountInfo>,
    /// Sender range proof account (optional, context state or record account).
    pub sender_range_proof_account: Option<&'a AccountInfo>,
    /// Recipient range proof account (optional, context state or record account).
    pub recipient_range_proof_account: Option<&'a AccountInfo>,
    /// The new source decryptable balance ciphertext after the transfer succeeds.
    pub new_source_decryptable_available_balance: PodAeCiphertext,
    /// Relative offset of the equality proof instruction, or 0 if using context state account.
    pub equality_proof_instruction_offset: i8,
    /// Relative offset of the transfer amount ciphertext validity proof instruction, or 0 if using context state account.
    pub transfer_amount_ciphertext_validity_proof_instruction_offset: i8,
    /// Relative offset of the sender range proof instruction, or 0 if using context state account.
    pub sender_range_proof_instruction_offset: i8,
    /// Relative offset of the recipient range proof instruction, or 0 if using context state account.
    pub recipient_range_proof_instruction_offset: i8,
}

impl Transfer<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Core accounts
        let mut account_metas = vec![
            AccountMeta::writable(self.source_token_account.key()),
            AccountMeta::writable(self.destination_token_account.key()),
            AccountMeta::readonly(self.mint.key()),
        ];
        let mut accounts = vec![
            self.source_token_account,
            self.destination_token_account,
            self.mint,
        ];

        // Check if sysvar is needed
        let sysvar_needed = self.equality_proof_instruction_offset != 0
            || self.transfer_amount_ciphertext_validity_proof_instruction_offset != 0
            || self.sender_range_proof_instruction_offset != 0
            || self.recipient_range_proof_instruction_offset != 0;

        if sysvar_needed {
            if let Some(sysvar_acc) = self.sysvar_instructions_account {
                account_metas.push(AccountMeta::readonly(sysvar_acc.key()));
                accounts.push(sysvar_acc);
            } else {
                // If proofs are inline (offset != 0), sysvar MUST be provided
                return Err(ProgramError::InvalidArgument); // Or a more specific error
            }
        }

        // Add optional proof accounts (Context State or Record)
        if let Some(proof_acc) = self.equality_proof_account {
            account_metas.push(AccountMeta::readonly(proof_acc.key()));
            accounts.push(proof_acc);
        }
        if let Some(proof_acc) = self.transfer_amount_ciphertext_validity_proof_account {
            account_metas.push(AccountMeta::readonly(proof_acc.key()));
            accounts.push(proof_acc);
        }
        if let Some(proof_acc) = self.sender_range_proof_account {
            account_metas.push(AccountMeta::readonly(proof_acc.key()));
            accounts.push(proof_acc);
        }
        if let Some(proof_acc) = self.recipient_range_proof_account {
            account_metas.push(AccountMeta::readonly(proof_acc.key()));
            accounts.push(proof_acc);
        }

        // Add authority and potential multisig signers
        account_metas.push(AccountMeta::readonly_signer(self.authority.key())); // Start assuming authority is signer
        accounts.push(self.authority);

        let authority_meta_index = account_metas.len() - 1;
        for multisig_signer in self.multisig_signers.iter() {
            // If multisig signers are present, authority is not a direct signer
            account_metas[authority_meta_index] = AccountMeta::readonly(self.authority.key());
            account_metas.push(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts.push(multisig_signer);
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 7 (Transfer)
        // -  [2..38]: new_source_decryptable_available_balance (36 bytes)
        // -  [38]: equality_proof_instruction_offset (1 byte, i8)
        // -  [39]: transfer_amount_ciphertext_validity_proof_instruction_offset (1 byte, i8)
        // -  [40]: sender_range_proof_instruction_offset (1 byte, i8)
        // -  [41]: recipient_range_proof_instruction_offset (1 byte, i8)
        const DATA_LEN: usize = 1 + 1 + POD_AE_CIPHERTEXT_LEN + 1 + 1 + 1 + 1; // 42 bytes
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];

        let balance_start = 2;
        let balance_end = balance_start + POD_AE_CIPHERTEXT_LEN;
        let eq_offset_start = balance_end;
        let valid_offset_start = eq_offset_start + 1;
        let send_offset_start = valid_offset_start + 1;
        let recip_offset_start = send_offset_start + 1;

        write_bytes(&mut instruction_data[0..1], &[27]); // Main discriminator
        write_bytes(&mut instruction_data[1..2], &[7]); // Transfer discriminator
        write_bytes(
            &mut instruction_data[balance_start..balance_end],
            &self.new_source_decryptable_available_balance.0,
        ); // Source balance ciphertext bytes
        write_bytes(
            &mut instruction_data[eq_offset_start..eq_offset_start + 1],
            &[self.equality_proof_instruction_offset as u8],
        ); // Equality offset
        write_bytes(
            &mut instruction_data[valid_offset_start..valid_offset_start + 1],
            &[self.transfer_amount_ciphertext_validity_proof_instruction_offset as u8],
        ); // Validity offset
        write_bytes(
            &mut instruction_data[send_offset_start..send_offset_start + 1],
            &[self.sender_range_proof_instruction_offset as u8],
        ); // Sender range offset
        write_bytes(
            &mut instruction_data[recip_offset_start..recip_offset_start + 1],
            &[self.recipient_range_proof_instruction_offset as u8],
        ); // Recipient range offset

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, DATA_LEN) },
        };

        invoke_signed(&instruction, &accounts, signers)
    }
}

/// Creates the CPI instruction for `TransferWithFee` based on the underlying
/// confidential transfer logic from the Token-2022 program.
///
/// Note: This wrapper corresponds to `ConfidentialTransferInstruction::TransferWithFee`.
/// Use this when the Mint has the `ConfidentialTransferFeeConfig` extension enabled.
///
/// The caller is responsible for managing the associated ZK proof instructions
/// (`VerifyCiphertextCommitmentEquality`, `VerifyTransferAmountValidityWithFee`,
/// `VerifyFeeSigma`, `VerifyFeeValidity`, `VerifyBatchedRangeProofU256`) or
/// context state accounts required by the Token-2022 program, ensuring they
/// are correctly placed relative to this instruction or provided via the
/// appropriate proof account fields.
pub struct TransferWithFee<'a> {
    /// The source SPL Token account.
    pub source_token_account: &'a AccountInfo,
    /// The destination SPL Token account.
    pub destination_token_account: &'a AccountInfo,
    /// The SPL Token mint (must have fee config). Marked writable as processor modifies withheld amounts.
    pub mint: &'a AccountInfo,
    /// The source account owner or delegate.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
    /// Instructions sysvar (optional, required if any proof offset is non-zero).
    pub sysvar_instructions_account: Option<&'a AccountInfo>,
    /// Equality proof account (optional, context state or record account).
    pub equality_proof_account: Option<&'a AccountInfo>,
    /// Transfer amount ciphertext validity proof account (optional, context state or record account).
    pub transfer_amount_ciphertext_validity_proof_account: Option<&'a AccountInfo>,
    /// Fee sigma proof account (optional, context state or record account).
    pub fee_sigma_proof_account: Option<&'a AccountInfo>,
    /// Fee ciphertext validity proof account (optional, context state or record account).
    pub fee_ciphertext_validity_proof_account: Option<&'a AccountInfo>,
    /// Range proof account (optional, context state or record account).
    pub range_proof_account: Option<&'a AccountInfo>,

    // Instruction Data fields incorporated into struct
    /// The new source decryptable balance ciphertext after the transfer succeeds.
    pub new_source_decryptable_available_balance: PodAeCiphertext,
    /// The transfer amount encrypted under the auditor ElGamal public key (low bits).
    pub transfer_amount_auditor_ciphertext_lo: PodElGamalCiphertext,
    /// The transfer amount encrypted under the auditor ElGamal public key (high bits).
    pub transfer_amount_auditor_ciphertext_hi: PodElGamalCiphertext,
    /// The fee commitment encrypted under the auditor ElGamal public key.
    pub fee_commitment_auditor_ciphertext: PodElGamalCiphertext,
    /// Relative offset of the equality proof instruction.
    pub equality_proof_instruction_offset: i8,
    /// Relative offset of the transfer amount ciphertext validity proof instruction.
    pub transfer_amount_ciphertext_validity_proof_instruction_offset: i8,
    /// Relative offset of the fee sigma proof instruction.
    pub fee_sigma_proof_instruction_offset: i8,
    /// Relative offset of the fee ciphertext validity proof instruction.
    pub fee_ciphertext_validity_proof_instruction_offset: i8,
    /// Relative offset of the range proof instruction.
    pub range_proof_instruction_offset: i8,
}

impl TransferWithFee<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Core accounts
        let mut account_metas = vec![
            AccountMeta::writable(self.source_token_account.key()),
            AccountMeta::writable(self.destination_token_account.key()),
            AccountMeta::writable(self.mint.key()), // Writable for fee updates
        ];
        let mut accounts = vec![
            self.source_token_account,
            self.destination_token_account,
            self.mint,
        ];

        // Check if sysvar is needed
        let sysvar_needed = self.equality_proof_instruction_offset != 0
            || self.transfer_amount_ciphertext_validity_proof_instruction_offset != 0
            || self.fee_sigma_proof_instruction_offset != 0
            || self.fee_ciphertext_validity_proof_instruction_offset != 0
            || self.range_proof_instruction_offset != 0;

        if sysvar_needed {
            if let Some(sysvar_acc) = self.sysvar_instructions_account {
                account_metas.push(AccountMeta::readonly(sysvar_acc.key()));
                accounts.push(sysvar_acc);
            } else {
                return Err(ProgramError::InvalidArgument); // Sysvar mandatory if any offset != 0
            }
        }

        // Add optional proof accounts
        if let Some(proof_acc) = self.equality_proof_account {
            account_metas.push(AccountMeta::readonly(proof_acc.key()));
            accounts.push(proof_acc);
        }
        if let Some(proof_acc) = self.transfer_amount_ciphertext_validity_proof_account {
            account_metas.push(AccountMeta::readonly(proof_acc.key()));
            accounts.push(proof_acc);
        }
        if let Some(proof_acc) = self.fee_sigma_proof_account {
            account_metas.push(AccountMeta::readonly(proof_acc.key()));
            accounts.push(proof_acc);
        }
        if let Some(proof_acc) = self.fee_ciphertext_validity_proof_account {
            account_metas.push(AccountMeta::readonly(proof_acc.key()));
            accounts.push(proof_acc);
        }
        if let Some(proof_acc) = self.range_proof_account {
            account_metas.push(AccountMeta::readonly(proof_acc.key()));
            accounts.push(proof_acc);
        }

        // Add authority and potential multisig signers
        account_metas.push(AccountMeta::readonly_signer(self.authority.key()));
        accounts.push(self.authority);

        let authority_meta_index = account_metas.len() - 1;
        for multisig_signer in self.multisig_signers.iter() {
            account_metas[authority_meta_index] = AccountMeta::readonly(self.authority.key());
            account_metas.push(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts.push(multisig_signer);
        }

        // Instruction data construction (Total 235 bytes)
        const ACTUAL_DATA_LEN: usize =
            1 + 1 + POD_AE_CIPHERTEXT_LEN + (3 * POD_ELGAMAL_CIPHERTEXT_LEN) + 5; // 2 + 36 + 3*64 + 5 = 235
        let mut instruction_data = [UNINIT_BYTE; ACTUAL_DATA_LEN];

        let balance_start = 2;
        let balance_end = balance_start + POD_AE_CIPHERTEXT_LEN; // 38
        let transfer_lo_start = balance_end;
        let transfer_lo_end = transfer_lo_start + POD_ELGAMAL_CIPHERTEXT_LEN; // 102
        let transfer_hi_start = transfer_lo_end;
        let transfer_hi_end = transfer_hi_start + POD_ELGAMAL_CIPHERTEXT_LEN; // 166
        let fee_commit_start = transfer_hi_end;
        let fee_commit_end = fee_commit_start + POD_ELGAMAL_CIPHERTEXT_LEN; // 230

        let eq_offset_idx = fee_commit_end; // 230
        let valid_offset_idx = eq_offset_idx + 1; // 231
        let fee_sigma_offset_idx = valid_offset_idx + 1; // 232
        let fee_valid_offset_idx = fee_sigma_offset_idx + 1; // 233
        let range_offset_idx = fee_valid_offset_idx + 1; // 234

        write_bytes(&mut instruction_data[0..1], &[27]); // Main discriminator
        write_bytes(&mut instruction_data[1..2], &[16]); // TransferWithFee discriminator

        write_bytes(
            &mut instruction_data[balance_start..balance_end],
            &self.new_source_decryptable_available_balance.0,
        );
        write_bytes(
            &mut instruction_data[transfer_lo_start..transfer_lo_end],
            &self.transfer_amount_auditor_ciphertext_lo.0,
        );
        write_bytes(
            &mut instruction_data[transfer_hi_start..transfer_hi_end],
            &self.transfer_amount_auditor_ciphertext_hi.0,
        );
        write_bytes(
            &mut instruction_data[fee_commit_start..fee_commit_end],
            &self.fee_commitment_auditor_ciphertext.0,
        );

        write_bytes(
            &mut instruction_data[eq_offset_idx..eq_offset_idx + 1],
            &[self.equality_proof_instruction_offset as u8],
        );
        write_bytes(
            &mut instruction_data[valid_offset_idx..valid_offset_idx + 1],
            &[self.transfer_amount_ciphertext_validity_proof_instruction_offset as u8],
        );
        write_bytes(
            &mut instruction_data[fee_sigma_offset_idx..fee_sigma_offset_idx + 1],
            &[self.fee_sigma_proof_instruction_offset as u8],
        );
        write_bytes(
            &mut instruction_data[fee_valid_offset_idx..fee_valid_offset_idx + 1],
            &[self.fee_ciphertext_validity_proof_instruction_offset as u8],
        );
        write_bytes(
            &mut instruction_data[range_offset_idx..range_offset_idx + 1],
            &[self.range_proof_instruction_offset as u8],
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, ACTUAL_DATA_LEN) },
        };

        invoke_signed(&instruction, &accounts, signers)
    }
}
