use core::mem::{self, MaybeUninit};
use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::extensions::ELGAMAL_PUBKEY_LEN;
use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

use super::{get_extension_from_bytes, DecryptableBalance, EncryptedBalance, Extension};

use super::{POD_AE_CIPHERTEXT_LEN, POD_ELGAMAL_CIPHERTEXT_LEN};

// State Structs and Extension Implementations

/// Confidential transfer mint configuration state mirroring SPL definition.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ConfidentialTransferMint {
    /// Authority to modify the `ConfidentialTransferMint` configuration and to
    /// approve new accounts (if `auto_approve_new_accounts` is true)
    pub authority: Pubkey, // Simplified from OptionalNonZeroPubkey

    /// Indicate if newly configured accounts must be approved by the
    /// `authority` before they may be used by the user.
    pub auto_approve_new_accounts: u8, // Simplified from PodBool

    /// Authority to decode any transfer amount in a confidential transfer.
    pub auditor_elgamal_pubkey: [u8; ELGAMAL_PUBKEY_LEN], // Simplified from OptionalNonZeroElGamalPubkey
}

impl Extension for ConfidentialTransferMint {
    const TYPE: super::ExtensionType = super::ExtensionType::ConfidentialTransferMint;
    const LEN: usize = mem::size_of::<Self>(); // Should be 65 bytes
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl ConfidentialTransferMint {
    /// The length of the `ConfidentialTransferMint` data.
    pub const LEN: usize = mem::size_of::<ConfidentialTransferMint>();

    /// Return a `ConfidentialTransferMint` from the given Mint account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&ConfidentialTransferMint, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(ProgramError::InvalidAccountData)
    }
}

/// Confidential account state mirroring SPL definition.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ConfidentialTransferAccount {
    /// `true` if this account has been approved for use.
    pub approved: u8, // Simplified from PodBool

    /// The public key associated with ElGamal encryption
    pub elgamal_pubkey: [u8; ELGAMAL_PUBKEY_LEN],

    /// The low 16 bits of the pending balance (encrypted by `elgamal_pubkey`)
    pub pending_balance_lo: EncryptedBalance,

    /// The high 48 bits of the pending balance (encrypted by `elgamal_pubkey`)
    pub pending_balance_hi: EncryptedBalance,

    /// The available balance (encrypted by `elgamal_pubkey`)
    pub available_balance: EncryptedBalance,

    /// The decryptable available balance
    pub decryptable_available_balance: DecryptableBalance,

    /// If `false`, the extended account rejects any incoming confidential transfers
    pub allow_confidential_credits: u8, // Simplified from PodBool

    /// If `false`, the base account rejects any incoming transfers
    pub allow_non_confidential_credits: u8, // Simplified from PodBool

    /// The total number of credits (`Deposit` or `Transfer`) to `pending_balance`
    pub pending_balance_credit_counter: [u8; 8], // Simplified from PodU64

    /// The maximum number of credits before `ApplyPendingBalance` is required
    pub maximum_pending_balance_credit_counter: [u8; 8], // Simplified from PodU64

    /// The `expected_pending_balance_credit_counter` from the last `ApplyPendingBalance`
    pub expected_pending_balance_credit_counter: [u8; 8], // Simplified from PodU64

    /// The actual `pending_balance_credit_counter` during the last `ApplyPendingBalance`
    pub actual_pending_balance_credit_counter: [u8; 8], // Simplified from PodU64
}

impl Extension for ConfidentialTransferAccount {
    const TYPE: super::ExtensionType = super::ExtensionType::ConfidentialTransferAccount;
    const LEN: usize = mem::size_of::<Self>(); // Should be 295 bytes
    const BASE_STATE: super::BaseState = super::BaseState::TokenAccount;
}

impl ConfidentialTransferAccount {
    /// The length of the `ConfidentialTransferAccount` data.
    pub const LEN: usize = mem::size_of::<ConfidentialTransferAccount>();

    /// Return a `ConfidentialTransferAccount` from the given Token account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.   
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&ConfidentialTransferAccount, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(ProgramError::InvalidAccountData)
    }
}

/// Confidential transfer fee extension data mirroring SPL definition.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ConfidentialTransferFeeConfig {
    /// Optional authority to set the withdraw withheld authority ElGamal key
    pub authority: Pubkey, // Simplified from OptionalNonZeroPubkey, assuming default Pubkey means None

    /// Withheld fees encrypted under this key
    pub withdraw_withheld_authority_elgamal_pubkey: [u8; ELGAMAL_PUBKEY_LEN],

    /// If `false`, harvest to mint is rejected.
    pub harvest_to_mint_enabled: u8, // Simplified from PodBool

    /// Withheld tokens moved to the mint for withdrawal.
    pub withheld_amount: EncryptedBalance, // Matches [u8; POD_ELGAMAL_CIPHERTEXT_LEN]
}

impl Extension for ConfidentialTransferFeeConfig {
    // Assuming ConfidentialTransferFee exists in the enum
    const TYPE: super::ExtensionType = super::ExtensionType::ConfidentialTransferFeeConfig;
    const LEN: usize = mem::size_of::<Self>(); // Expected: 32 + 32 + 1 + 64 = 129 bytes
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl ConfidentialTransferFeeConfig {
    /// The length of the `ConfidentialTransferFeeConfig` data.
    pub const LEN: usize = mem::size_of::<ConfidentialTransferFeeConfig>();

    /// Return a `ConfidentialTransferFeeConfig` from the given Mint account info.
    #[inline(always)]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<&ConfidentialTransferFeeConfig, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(ProgramError::InvalidAccountData)
    }
}

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
    pub auditor_elgamal_pubkey: Option<&'a [u8; ELGAMAL_PUBKEY_LEN]>,
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
    pub auditor_elgamal_pubkey: Option<&'a [u8; ELGAMAL_PUBKEY_LEN]>,
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

// Modify ConfigureAccount to use const generics and MaybeUninit
pub struct ConfigureAccount<'a, const ACCOUNTS_LEN: usize> {
    /// Token account to configure.
    pub token_account: &'a AccountInfo,
    /// Mint associated with the token account.
    pub mint: &'a AccountInfo,
    /// Token account owner.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
    /// The ElGamal public key for the account.
    pub elgamal_pk: [u8; ELGAMAL_PUBKEY_LEN],
    /// The decryptable balance (typically ciphertext corresponding to 0)
    /// encrypted with the `elgamal_pk`.
    pub decryptable_zero_balance: &'a DecryptableBalance,
}

impl<const ACCOUNTS_LEN: usize> ConfigureAccount<'_, ACCOUNTS_LEN> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Calculate minimum required ACCOUNTS_LEN
        let min_accounts = 3 + self.multisig_signers.len();
        if ACCOUNTS_LEN < min_accounts {
            return Err(ProgramError::InvalidArgument);
        }

        // Create arrays of MaybeUninit
        const UNINIT_ACC_METAS: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut account_metas = [UNINIT_ACC_METAS; ACCOUNTS_LEN];

        let mut accounts: [MaybeUninit<&AccountInfo>; ACCOUNTS_LEN] =
            [MaybeUninit::uninit(); ACCOUNTS_LEN];

        // Track the current index
        let mut idx = 0;

        // Base accounts
        account_metas[idx].write(AccountMeta::writable(self.token_account.key()));
        accounts[idx].write(self.token_account);
        idx += 1;

        account_metas[idx].write(AccountMeta::readonly(self.mint.key()));
        accounts[idx].write(self.mint);
        idx += 1;

        // Add authority (starts assuming it's a signer)
        let authority_idx = idx;
        account_metas[idx].write(AccountMeta::readonly_signer(self.authority.key()));
        accounts[idx].write(self.authority);
        idx += 1;

        // Add optional multisig signers
        for multisig_signer in self.multisig_signers {
            // If multisig signers are present, authority is not a direct signer
            if idx == authority_idx + 1 {
                account_metas[authority_idx].write(AccountMeta::readonly(self.authority.key()));
            }

            account_metas[idx].write(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts[idx].write(multisig_signer);
            idx += 1;
        }

        // Create slices from the initialized parts
        let account_metas_slice = unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, idx)
        };

        let acc_infos: [&AccountInfo; ACCOUNTS_LEN] = unsafe {
            core::slice::from_raw_parts(accounts.as_ptr() as *const &AccountInfo, ACCOUNTS_LEN)
                .try_into()
                .unwrap()
        };

        // Instruction data layout:
        const DATA_LEN: usize = 1 + 1 + ELGAMAL_PUBKEY_LEN + POD_AE_CIPHERTEXT_LEN; // 1 + 1 + 32 + 36 = 70
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];

        write_bytes(&mut instruction_data[0..1], &[27]); // Main discriminator
        write_bytes(&mut instruction_data[1..2], &[2]); // ConfigureAccount discriminator
        write_bytes(
            &mut instruction_data[2..(2 + ELGAMAL_PUBKEY_LEN)],
            &self.elgamal_pk,
        ); // ElGamal PK bytes
        write_bytes(
            &mut instruction_data[(2 + ELGAMAL_PUBKEY_LEN)..DATA_LEN],
            &self.decryptable_zero_balance.0,
        ); // Decryptable zero balance ciphertext bytes

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: account_metas_slice,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, DATA_LEN) },
        };

        invoke_signed(&instruction, &acc_infos, signers)
    }
}

pub struct ConfigureAccountWithRegistry<'a, const ACCOUNTS_LEN: usize> {
    /// Token account to configure.
    pub token_account: &'a AccountInfo,
    /// Mint associated with the token account.
    pub mint: &'a AccountInfo,
    /// ElGamal registry account containing the ElGamal public key.
    pub elgamal_registry_account: &'a AccountInfo,
    /// Optional payer account for reallocation if the token account is too small.
    pub payer: Option<&'a AccountInfo>,
    /// Optional system program account. Required if payer is Some for reallocation.
    pub system_program: Option<&'a AccountInfo>,
}

impl<const ACCOUNTS_LEN: usize> ConfigureAccountWithRegistry<'_, ACCOUNTS_LEN> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Calculate minimum required ACCOUNTS_LEN
        let min_accounts = 3 + if self.payer.is_some() { 2 } else { 0 };
        if ACCOUNTS_LEN < min_accounts {
            return Err(ProgramError::InvalidArgument);
        }

        const UNINIT_ACC_METAS: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut account_metas = [UNINIT_ACC_METAS; ACCOUNTS_LEN];

        let mut accounts: [MaybeUninit<&AccountInfo>; ACCOUNTS_LEN] =
            [MaybeUninit::uninit(); ACCOUNTS_LEN];

        // Track the current index
        let mut idx = 0;

        // Base accounts
        account_metas[idx].write(AccountMeta::writable(self.token_account.key()));
        accounts[idx].write(self.token_account);
        idx += 1;

        account_metas[idx].write(AccountMeta::readonly(self.mint.key()));
        accounts[idx].write(self.mint);
        idx += 1;

        account_metas[idx].write(AccountMeta::readonly(self.elgamal_registry_account.key()));
        accounts[idx].write(self.elgamal_registry_account);
        idx += 1;

        // Optional payer and system program
        if let Some(payer_info) = self.payer {
            if let Some(system_program_info) = self.system_program {
                account_metas[idx].write(AccountMeta::writable_signer(payer_info.key()));
                accounts[idx].write(payer_info);
                idx += 1;

                account_metas[idx].write(AccountMeta::readonly(system_program_info.key()));
                accounts[idx].write(system_program_info);
                idx += 1;
            } else {
                return Err(ProgramError::InvalidArgument);
            }
        }

        // Create slices from the initialized parts
        let account_metas_slice = unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, idx)
        };

        let acc_infos: [&AccountInfo; ACCOUNTS_LEN] = unsafe {
            core::slice::from_raw_parts(accounts.as_ptr() as *const &AccountInfo, ACCOUNTS_LEN)
                .try_into()
                .unwrap()
        };

        // Instruction data layout:
        let mut instruction_data = [UNINIT_BYTE; 2];
        write_bytes(&mut instruction_data[0..1], &[27]);
        write_bytes(&mut instruction_data[1..2], &[14]);

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: account_metas_slice,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 2) },
        };

        invoke_signed(&instruction, &acc_infos, signers)
    }
}

pub struct ApproveAccount<'a, const ACCOUNTS_LEN: usize> {
    /// The SPL Token account to approve.
    pub token_account: &'a AccountInfo,
    /// The SPL Token mint.
    pub mint: &'a AccountInfo,
    /// Confidential transfer mint authority.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
}

impl<const ACCOUNTS_LEN: usize> ApproveAccount<'_, ACCOUNTS_LEN> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Calculate minimum required ACCOUNTS_LEN
        let min_accounts = 3 + self.multisig_signers.len();
        if ACCOUNTS_LEN < min_accounts {
            return Err(ProgramError::InvalidArgument);
        }

        // Create arrays of MaybeUninit
        const UNINIT_ACC_METAS: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut account_metas = [UNINIT_ACC_METAS; ACCOUNTS_LEN];

        let mut accounts: [MaybeUninit<&AccountInfo>; ACCOUNTS_LEN] =
            [MaybeUninit::uninit(); ACCOUNTS_LEN];

        // Track the current index
        let mut idx = 0;

        // Base accounts
        account_metas[idx].write(AccountMeta::writable(self.token_account.key()));
        accounts[idx].write(self.token_account);
        idx += 1;

        account_metas[idx].write(AccountMeta::readonly(self.mint.key()));
        accounts[idx].write(self.mint);
        idx += 1;

        // Add authority (starts assuming it's a signer)
        let authority_idx = idx;
        account_metas[idx].write(AccountMeta::readonly_signer(self.authority.key()));
        accounts[idx].write(self.authority);
        idx += 1;

        // Add optional multisig signers
        for multisig_signer in self.multisig_signers {
            // If multisig signers are present, authority is not a direct signer
            if idx == authority_idx + 1 {
                account_metas[authority_idx].write(AccountMeta::readonly(self.authority.key()));
            }

            account_metas[idx].write(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts[idx].write(multisig_signer);
            idx += 1;
        }

        // Create slices from the initialized parts
        let account_metas_slice = unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, idx)
        };

        let acc_infos: [&AccountInfo; ACCOUNTS_LEN] = unsafe {
            core::slice::from_raw_parts(accounts.as_ptr() as *const &AccountInfo, ACCOUNTS_LEN)
                .try_into()
                .unwrap()
        };

        // Instruction data layout:
        let mut instruction_data = [UNINIT_BYTE; 2];
        write_bytes(&mut instruction_data[0..1], &[27]);
        write_bytes(&mut instruction_data[1..2], &[3]);

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: account_metas_slice,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 2) },
        };

        invoke_signed(&instruction, &acc_infos, signers)
    }
}

pub struct EmptyAccount<'a, const ACCOUNTS_LEN: usize> {
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

impl<const ACCOUNTS_LEN: usize> EmptyAccount<'_, ACCOUNTS_LEN> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Count required accounts
        let mut required_accounts = 3; // token_account, proof_account, authority

        // Add optional record account if needed
        required_accounts +=
            (self.proof_instruction_offset != 0 && self.record_account.is_some()) as usize;

        // Add multisig signers
        required_accounts += self.multisig_signers.len();

        if required_accounts != ACCOUNTS_LEN {
            return Err(ProgramError::InvalidArgument);
        }

        // Account metas
        const UNINIT_ACC_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut account_metas = [UNINIT_ACC_META; ACCOUNTS_LEN];

        // Account infos
        const UNINIT_ACC_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::<&AccountInfo>::uninit();
        let mut accounts = [UNINIT_ACC_INFO; ACCOUNTS_LEN];

        // Initialize accounts
        let mut idx = 0;

        // Token account
        account_metas[idx].write(AccountMeta::writable(self.token_account.key()));
        accounts[idx].write(self.token_account);
        idx += 1;

        // Proof account
        account_metas[idx].write(AccountMeta::readonly(self.proof_account.key()));
        accounts[idx].write(self.proof_account);
        idx += 1;

        // Optional record account
        if self.proof_instruction_offset != 0 && self.record_account.is_some() {
            account_metas[idx].write(AccountMeta::readonly(self.record_account.unwrap().key()));
            accounts[idx].write(self.record_account.unwrap());
            idx += 1;
        }

        // Authority
        if self.multisig_signers.is_empty() {
            account_metas[idx].write(AccountMeta::readonly_signer(self.authority.key()));
        } else {
            account_metas[idx].write(AccountMeta::readonly(self.authority.key()));
        }
        accounts[idx].write(self.authority);
        idx += 1;

        // Multisig signers
        for (i, signer) in self.multisig_signers.iter().enumerate() {
            account_metas[idx + i].write(AccountMeta::readonly_signer(signer.key()));
            accounts[idx + i].write(signer);
        }

        // Convert to slices safely
        let acc_metas = unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, ACCOUNTS_LEN)
        };

        let acc_infos: [&AccountInfo; ACCOUNTS_LEN] = unsafe {
            core::slice::from_raw_parts(accounts.as_ptr() as *const &AccountInfo, ACCOUNTS_LEN)
                .try_into()
                .unwrap() // Safe as we verified the length
        };

        // Instruction data construction
        const DATA_LEN: usize = 3;
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];

        // Set main discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[27]);
        // Set EmptyAccount discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[4]);
        // Set proof_instruction_offset as u8 at offset [2]
        write_bytes(
            &mut instruction_data[2..3],
            &[self.proof_instruction_offset as u8],
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: acc_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, DATA_LEN) },
        };

        invoke_signed(&instruction, &acc_infos, signers)
    }
}

pub struct Deposit<'a, const ACCOUNTS_LEN: usize> {
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

impl<const ACCOUNTS_LEN: usize> Deposit<'_, ACCOUNTS_LEN> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Calculate minimum required ACCOUNTS_LEN
        let min_accounts = 3 + self.multisig_signers.len();
        if ACCOUNTS_LEN < min_accounts {
            return Err(ProgramError::InvalidArgument);
        }

        // Create arrays of MaybeUninit
        const UNINIT_ACC_METAS: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut account_metas = [UNINIT_ACC_METAS; ACCOUNTS_LEN];
        let mut accounts: [MaybeUninit<&AccountInfo>; ACCOUNTS_LEN] =
            [MaybeUninit::uninit(); ACCOUNTS_LEN];

        // Track the current index
        let mut idx = 0;

        // Base accounts
        account_metas[idx].write(AccountMeta::writable(self.token_account.key()));
        accounts[idx].write(self.token_account);
        idx += 1;

        account_metas[idx].write(AccountMeta::readonly(self.mint.key()));
        accounts[idx].write(self.mint);
        idx += 1;

        // Add authority (starts assuming it's a signer)
        let authority_idx = idx;
        account_metas[idx].write(AccountMeta::readonly_signer(self.authority.key()));
        accounts[idx].write(self.authority);
        idx += 1;

        // Add optional multisig signers
        for multisig_signer in self.multisig_signers {
            // If multisig signers are present, authority is not a direct signer
            if idx == authority_idx + 1 {
                account_metas[authority_idx].write(AccountMeta::readonly(self.authority.key()));
            }

            account_metas[idx].write(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts[idx].write(multisig_signer);
            idx += 1;
        }

        // Create slices from the initialized parts
        let account_metas_slice = unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, idx)
        };

        let acc_infos: [&AccountInfo; ACCOUNTS_LEN] = unsafe {
            core::slice::from_raw_parts(accounts.as_ptr() as *const &AccountInfo, ACCOUNTS_LEN)
                .try_into()
                .unwrap() // this is safe as we know the length of the array
        };

        // Instruction data layout:
        let mut instruction_data = [UNINIT_BYTE; 11];
        write_bytes(&mut instruction_data[0..1], &[27]); // Main discriminator
        write_bytes(&mut instruction_data[1..2], &[5]); // Deposit discriminator
        write_bytes(&mut instruction_data[2..10], &self.amount.to_le_bytes()); // Amount
        write_bytes(&mut instruction_data[10..11], &[self.decimals]); // Decimals

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: account_metas_slice,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 11) },
        };

        invoke_signed(&instruction, &acc_infos, signers)
    }
}

pub struct Withdraw<'a, const ACCOUNTS_LEN: usize> {
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
    pub new_decryptable_available_balance: [u8; POD_AE_CIPHERTEXT_LEN],
    /// Relative offset of the equality proof instruction, or 0 if using context state account.
    pub equality_proof_instruction_offset: i8,
    /// Relative offset of the range proof instruction, or 0 if using context state account.
    pub range_proof_instruction_offset: i8,
}

impl<const ACCOUNTS_LEN: usize> Withdraw<'_, ACCOUNTS_LEN> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Count required accounts
        let mut required_accounts = 3; // token_account, mint, authority

        // Add optional accounts
        if self.equality_proof_instruction_offset != 0 || self.range_proof_instruction_offset != 0 {
            required_accounts += self.sysvar_instructions_account.is_some() as usize;
        }

        required_accounts += self.equality_proof_account.is_some() as usize;
        required_accounts += self.range_proof_account.is_some() as usize;
        required_accounts += self.multisig_signers.len();

        if required_accounts != ACCOUNTS_LEN {
            return Err(ProgramError::InvalidArgument);
        }

        // Account metas
        const UNINIT_ACC_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut account_metas = [UNINIT_ACC_META; ACCOUNTS_LEN];

        // Account infos
        const UNINIT_ACC_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::<&AccountInfo>::uninit();
        let mut accounts = [UNINIT_ACC_INFO; ACCOUNTS_LEN];

        // Basic accounts (first three are always present)
        let mut idx = 0;

        account_metas[idx].write(AccountMeta::writable(self.token_account.key()));
        accounts[idx].write(self.token_account);
        idx += 1;

        account_metas[idx].write(AccountMeta::readonly(self.mint.key()));
        accounts[idx].write(self.mint);
        idx += 1;

        // Authority
        if self.multisig_signers.is_empty() {
            account_metas[idx].write(AccountMeta::readonly_signer(self.authority.key()));
        } else {
            account_metas[idx].write(AccountMeta::readonly(self.authority.key()));
        }
        accounts[idx].write(self.authority);
        idx += 1;

        // Optional sysvar account
        if (self.equality_proof_instruction_offset != 0 || self.range_proof_instruction_offset != 0)
            && self.sysvar_instructions_account.is_some()
        {
            account_metas[idx].write(AccountMeta::readonly(
                self.sysvar_instructions_account.unwrap().key(),
            ));
            accounts[idx].write(self.sysvar_instructions_account.unwrap());
            idx += 1;
        }

        // Optional proof accounts
        if let Some(acc) = self.equality_proof_account {
            account_metas[idx].write(AccountMeta::readonly(acc.key()));
            accounts[idx].write(acc);
            idx += 1;
        }

        if let Some(acc) = self.range_proof_account {
            account_metas[idx].write(AccountMeta::readonly(acc.key()));
            accounts[idx].write(acc);
            idx += 1;
        }

        // Multisig signers
        for (i, signer) in self.multisig_signers.iter().enumerate() {
            account_metas[idx + i].write(AccountMeta::readonly_signer(signer.key()));
            accounts[idx + i].write(signer);
        }

        // Convert to slices safely
        let acc_metas = unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, ACCOUNTS_LEN)
        };

        let acc_infos: [&AccountInfo; ACCOUNTS_LEN] = unsafe {
            core::slice::from_raw_parts(accounts.as_ptr() as *const &AccountInfo, ACCOUNTS_LEN)
                .try_into()
                .unwrap() // Safe as we verified the length
        };

        // Instruction data construction
        const DATA_LEN: usize = 1 + 1 + 8 + 1 + POD_AE_CIPHERTEXT_LEN + 2;
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];

        // Set main discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[27]);
        // Set Withdraw discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[4]);
        // Set amount as u64 at offset [2..10]
        write_bytes(&mut instruction_data[2..10], &self.amount.to_le_bytes());
        // Set decimals as u8 at offset [10..11]
        write_bytes(&mut instruction_data[10..11], &[self.decimals]);
        // Set new_decryptable_available_balance as [u8; POD_AE_CIPHERTEXT_LEN] at offset [11..47]
        write_bytes(
            &mut instruction_data[11..47],
            &self.new_decryptable_available_balance,
        );
        // Set proof instruction offsets at offset [47..49]
        write_bytes(
            &mut instruction_data[47..49],
            &[
                self.equality_proof_instruction_offset as u8,
                self.range_proof_instruction_offset as u8,
            ],
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: acc_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, DATA_LEN) },
        };

        invoke_signed(&instruction, &acc_infos, signers)
    }
}

pub struct ApplyPendingBalance<'a, const ACCOUNTS_LEN: usize> {
    /// The SPL Token account holding the pending balance.
    pub token_account: &'a AccountInfo,
    /// The account owner.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
    /// The expected number of pending balance credits to apply.
    pub expected_pending_balance_credit_counter: u64,
    /// The new decryptable balance ciphertext after applying the pending balance.
    pub new_decryptable_available_balance: [u8; POD_AE_CIPHERTEXT_LEN],
}

impl<const ACCOUNTS_LEN: usize> ApplyPendingBalance<'_, ACCOUNTS_LEN> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let required_accounts = 2 + self.multisig_signers.len();

        if ACCOUNTS_LEN != required_accounts {
            return Err(ProgramError::InvalidArgument);
        }

        let mut account_metas: [MaybeUninit<AccountMeta>; ACCOUNTS_LEN] =
            unsafe { MaybeUninit::uninit().assume_init() };
        const UNINIT_ACC_INFOS: MaybeUninit<&AccountInfo> = MaybeUninit::<&AccountInfo>::uninit();
        let mut accounts = [UNINIT_ACC_INFOS; ACCOUNTS_LEN];

        account_metas[0].write(AccountMeta::writable(self.token_account.key()));
        accounts[0].write(self.token_account);

        account_metas[1].write(AccountMeta::readonly_signer(self.authority.key()));
        accounts[1].write(self.authority);

        if !self.multisig_signers.is_empty() {
            account_metas[1].write(AccountMeta::readonly(self.authority.key()));
            for (i, multisig_signer) in self.multisig_signers.iter().enumerate() {
                account_metas[2 + i].write(AccountMeta::readonly_signer(multisig_signer.key()));
                accounts[2 + i].write(multisig_signer);
            }
        }

        let acc_metas: &[AccountMeta] = unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, ACCOUNTS_LEN)
        };

        let acc_infos: [&AccountInfo; ACCOUNTS_LEN] = unsafe {
            core::slice::from_raw_parts(accounts.as_ptr() as *const &AccountInfo, ACCOUNTS_LEN)
                .try_into()
                .unwrap()
        };

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 8 (ApplyPendingBalance)
        // -  [2..10]: expected_pending_balance_credit_counter (8 bytes, u64)
        // -  [10..46]: new_decryptable_available_balance (36 bytes, [u8; POD_AE_CIPHERTEXT_LEN])
        const DATA_LEN: usize = 1 + 1 + 8 + POD_AE_CIPHERTEXT_LEN;
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];

        // Set main discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[27]);
        // Set ApplyPendingBalance discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[8]);
        // Set expected_pending_balance_credit_counter as u64 at offset [2..10]
        write_bytes(
            &mut instruction_data[2..10],
            &self.expected_pending_balance_credit_counter.to_le_bytes(),
        );
        // Set new_decryptable_available_balance as [u8; POD_AE_CIPHERTEXT_LEN] at offset [10..DATA_LEN]
        write_bytes(
            &mut instruction_data[10..DATA_LEN],
            &self.new_decryptable_available_balance,
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: acc_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, DATA_LEN) },
        };

        invoke_signed(&instruction, &acc_infos, signers)
    }
}

pub struct DisableConfidentialCredits<'a, const ACCOUNTS_LEN: usize> {
    /// The SPL Token account to disable confidential credits for.
    pub token_account: &'a AccountInfo,
    /// The account owner or delegate.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
}

impl<const ACCOUNTS_LEN: usize> DisableConfidentialCredits<'_, ACCOUNTS_LEN> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let required_accounts = 2 + self.multisig_signers.len();

        if ACCOUNTS_LEN != required_accounts {
            return Err(ProgramError::InvalidArgument);
        }

        let mut account_metas: [MaybeUninit<AccountMeta>; ACCOUNTS_LEN] =
            unsafe { MaybeUninit::uninit().assume_init() };
        const UNINIT_ACC_INFOS: MaybeUninit<&AccountInfo> = MaybeUninit::<&AccountInfo>::uninit();
        let mut accounts = [UNINIT_ACC_INFOS; ACCOUNTS_LEN];

        account_metas[0].write(AccountMeta::writable(self.token_account.key()));
        accounts[0].write(self.token_account);

        account_metas[1].write(AccountMeta::readonly_signer(self.authority.key()));
        accounts[1].write(self.authority);

        if !self.multisig_signers.is_empty() {
            account_metas[1].write(AccountMeta::readonly(self.authority.key()));
            for (i, multisig_signer) in self.multisig_signers.iter().enumerate() {
                account_metas[2 + i].write(AccountMeta::readonly_signer(multisig_signer.key()));
                accounts[2 + i].write(multisig_signer);
            }
        }

        let acc_metas: &[AccountMeta] = unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, ACCOUNTS_LEN)
        };

        let acc_infos: [&AccountInfo; ACCOUNTS_LEN] = unsafe {
            core::slice::from_raw_parts(accounts.as_ptr() as *const &AccountInfo, ACCOUNTS_LEN)
                .try_into()
                .unwrap()
        };

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 11 (DisableConfidentialCredits)
        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: acc_metas,
            data: &[27, 11],
        };

        invoke_signed(&instruction, &acc_infos, signers)
    }
}

pub struct DisableNonConfidentialCredits<'a, const ACCOUNTS_LEN: usize> {
    /// The SPL Token account to disable non-confidential credits for.
    pub token_account: &'a AccountInfo,
    /// The account owner or delegate.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
}

impl<const ACCOUNTS_LEN: usize> DisableNonConfidentialCredits<'_, ACCOUNTS_LEN> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let required_accounts = 2 + self.multisig_signers.len();

        if ACCOUNTS_LEN != required_accounts {
            return Err(ProgramError::InvalidArgument);
        }

        let mut account_metas: [MaybeUninit<AccountMeta>; ACCOUNTS_LEN] =
            unsafe { MaybeUninit::uninit().assume_init() };
        const UNINIT_ACC_INFOS: MaybeUninit<&AccountInfo> = MaybeUninit::<&AccountInfo>::uninit();
        let mut accounts = [UNINIT_ACC_INFOS; ACCOUNTS_LEN];

        account_metas[0].write(AccountMeta::writable(self.token_account.key()));
        accounts[0].write(self.token_account);

        account_metas[1].write(AccountMeta::readonly_signer(self.authority.key()));
        accounts[1].write(self.authority);

        if !self.multisig_signers.is_empty() {
            account_metas[1].write(AccountMeta::readonly(self.authority.key()));
            for (i, multisig_signer) in self.multisig_signers.iter().enumerate() {
                account_metas[2 + i].write(AccountMeta::readonly_signer(multisig_signer.key()));
                accounts[2 + i].write(multisig_signer);
            }
        }

        let acc_metas: &[AccountMeta] = unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, ACCOUNTS_LEN)
        };

        let acc_infos: [&AccountInfo; ACCOUNTS_LEN] = unsafe {
            core::slice::from_raw_parts(accounts.as_ptr() as *const &AccountInfo, ACCOUNTS_LEN)
                .try_into()
                .unwrap()
        };

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 12 (DisableNonConfidentialCredits)
        let instruction_data = &[27u8, 12u8];

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: acc_metas,
            data: instruction_data,
        };

        invoke_signed(&instruction, &acc_infos, signers)
    }
}

pub struct EnableConfidentialCredits<'a, const ACCOUNTS_LEN: usize> {
    /// The SPL Token account to enable confidential credits for.
    pub token_account: &'a AccountInfo,
    /// The account owner or delegate.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
}

impl<const ACCOUNTS_LEN: usize> EnableConfidentialCredits<'_, ACCOUNTS_LEN> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let required_accounts = 2 + self.multisig_signers.len();

        if ACCOUNTS_LEN != required_accounts {
            return Err(ProgramError::InvalidArgument);
        }

        let mut account_metas: [MaybeUninit<AccountMeta>; ACCOUNTS_LEN] =
            unsafe { MaybeUninit::uninit().assume_init() };
        const UNINIT_ACC_INFOS: MaybeUninit<&AccountInfo> = MaybeUninit::<&AccountInfo>::uninit();
        let mut accounts = [UNINIT_ACC_INFOS; ACCOUNTS_LEN];

        account_metas[0].write(AccountMeta::writable(self.token_account.key()));
        accounts[0].write(self.token_account);

        account_metas[1].write(AccountMeta::readonly_signer(self.authority.key()));
        accounts[1].write(self.authority);

        if !self.multisig_signers.is_empty() {
            account_metas[1].write(AccountMeta::readonly(self.authority.key()));
            for (i, multisig_signer) in self.multisig_signers.iter().enumerate() {
                account_metas[2 + i].write(AccountMeta::readonly_signer(multisig_signer.key()));
                accounts[2 + i].write(multisig_signer);
            }
        }

        let acc_metas: &[AccountMeta] = unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, ACCOUNTS_LEN)
        };

        let acc_infos: [&AccountInfo; ACCOUNTS_LEN] = unsafe {
            core::slice::from_raw_parts(accounts.as_ptr() as *const &AccountInfo, ACCOUNTS_LEN)
                .try_into()
                .unwrap()
        };

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 15 (EnableNonConfidentialCredits)
        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: acc_metas, // Slice is fine here
            data: &[27, 13],
        };

        invoke_signed(&instruction, &acc_infos, signers)
    }
}

pub struct EnableNonConfidentialCredits<'a, const ACCOUNTS_LEN: usize> {
    /// The SPL Token account to enable non-confidential credits for.
    pub token_account: &'a AccountInfo,
    /// The account owner or delegate.
    pub authority: &'a AccountInfo,
    /// Optional multisig signers if the authority is a multisig account.
    pub multisig_signers: &'a [&'a AccountInfo],
}

impl<const ACCOUNTS_LEN: usize> EnableNonConfidentialCredits<'_, ACCOUNTS_LEN> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let required_accounts = 2 + self.multisig_signers.len();
        if ACCOUNTS_LEN != required_accounts {
            return Err(ProgramError::InvalidArgument);
        }

        let mut account_metas: [MaybeUninit<AccountMeta>; ACCOUNTS_LEN] =
            unsafe { MaybeUninit::uninit().assume_init() };
        const UNINIT_ACC_INFOS: MaybeUninit<&AccountInfo> = MaybeUninit::<&AccountInfo>::uninit();
        let mut accounts = [UNINIT_ACC_INFOS; ACCOUNTS_LEN];

        account_metas[0].write(AccountMeta::writable(self.token_account.key()));
        accounts[0].write(self.token_account);

        account_metas[1].write(AccountMeta::readonly_signer(self.authority.key()));
        accounts[1].write(self.authority);

        if !self.multisig_signers.is_empty() {
            account_metas[1].write(AccountMeta::readonly(self.authority.key()));
            for (i, multisig_signer) in self.multisig_signers.iter().enumerate() {
                account_metas[2 + i].write(AccountMeta::readonly_signer(multisig_signer.key()));
                accounts[2 + i].write(multisig_signer);
            }
        }

        let acc_metas: &[AccountMeta] = unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, ACCOUNTS_LEN)
        };

        let acc_infos: [&AccountInfo; ACCOUNTS_LEN] = unsafe {
            core::slice::from_raw_parts(accounts.as_ptr() as *const &AccountInfo, ACCOUNTS_LEN)
                .try_into()
                .unwrap()
        };

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8) -> 27 (ConfidentialTransferExtension)
        // -  [1]: extension instruction discriminator (1 byte, u8) -> 15 (EnableNonConfidentialCredits)
        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: acc_metas,
            data: &[27, 15],
        };

        invoke_signed(&instruction, &acc_infos, signers)
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
pub struct Transfer<'a, const ACCOUNTS_LEN: usize> {
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
    /// Range proof account (optional, context state or record account).
    pub range_proof_account: Option<&'a AccountInfo>,
    /// The new source decryptable balance ciphertext after the transfer succeeds.
    pub new_source_decryptable_available_balance: [u8; POD_AE_CIPHERTEXT_LEN],
    /// Relative offset of the equality proof instruction, or 0 if using context state account.
    pub equality_proof_instruction_offset: i8,
    /// Relative offset of the transfer amount ciphertext validity proof instruction, or 0 if using context state account.
    pub transfer_amount_ciphertext_validity_proof_instruction_offset: i8,
    /// Relative offset of the range proof instruction, or 0 if using context state account.
    pub range_proof_instruction_offset: i8,
}

impl<const ACCOUNTS_LEN: usize> Transfer<'_, ACCOUNTS_LEN> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // source_account, destination_account, mint
        let base_accounts = 3;

        // Check if sysvar is needed based on any non-zero proof offset
        let sysvar_needed = self.equality_proof_instruction_offset != 0
            || self.transfer_amount_ciphertext_validity_proof_instruction_offset != 0
            || self.range_proof_instruction_offset != 0;

        let sysvar_count = if sysvar_needed {
            self.sysvar_instructions_account.is_some() as usize
        } else {
            0
        };

        // Count optional proof accounts that are Some
        let proof_accounts_count = [
            self.equality_proof_account,
            self.transfer_amount_ciphertext_validity_proof_account,
            self.range_proof_account, // Only one range proof account
        ]
        .iter()
        .filter(|&&opt| opt.is_some())
        .count();

        let authority_accounts_count = 1 + self.multisig_signers.len();

        let required_accounts =
            base_accounts + sysvar_count + proof_accounts_count + authority_accounts_count;

        if ACCOUNTS_LEN != required_accounts {
            return Err(ProgramError::InvalidArgument);
        }

        let mut account_metas: [MaybeUninit<AccountMeta>; ACCOUNTS_LEN] =
            unsafe { MaybeUninit::uninit().assume_init() };
        const UNINIT_ACC_INFOS: MaybeUninit<&AccountInfo> = MaybeUninit::<&AccountInfo>::uninit();
        let mut accounts = [UNINIT_ACC_INFOS; ACCOUNTS_LEN];

        let mut idx = 0;

        account_metas[idx].write(AccountMeta::writable(self.source_token_account.key()));
        accounts[idx].write(self.source_token_account);
        idx += 1;

        account_metas[idx].write(AccountMeta::writable(self.destination_token_account.key()));
        accounts[idx].write(self.destination_token_account);
        idx += 1;

        account_metas[idx].write(AccountMeta::readonly(self.mint.key()));
        accounts[idx].write(self.mint);
        idx += 1;

        if sysvar_needed {
            if let Some(sysvar_acc) = self.sysvar_instructions_account {
                account_metas[idx].write(AccountMeta::readonly(sysvar_acc.key()));
                accounts[idx].write(sysvar_acc);
                idx += 1;
            } else {
                return Err(ProgramError::InvalidArgument);
            }
        }

        let proof_accounts_options = [
            self.equality_proof_account,
            self.transfer_amount_ciphertext_validity_proof_account,
            self.range_proof_account,
        ];
        for proof_acc in proof_accounts_options.into_iter().flatten() {
            account_metas[idx].write(AccountMeta::readonly(proof_acc.key()));
            accounts[idx].write(proof_acc);
            idx += 1;
        }

        let is_authority_signer = self.multisig_signers.is_empty();
        let authority_meta = if is_authority_signer {
            AccountMeta::readonly_signer(self.authority.key())
        } else {
            AccountMeta::readonly(self.authority.key())
        };
        account_metas[idx].write(authority_meta);
        accounts[idx].write(self.authority);
        idx += 1;

        for multisig_signer in self.multisig_signers {
            account_metas[idx].write(AccountMeta::readonly_signer(multisig_signer.key()));
            accounts[idx].write(multisig_signer);
            idx += 1;
        }

        let acc_metas: &[AccountMeta] = unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, ACCOUNTS_LEN)
        };

        let acc_infos: [&AccountInfo; ACCOUNTS_LEN] = unsafe {
            core::slice::from_raw_parts(accounts.as_ptr() as *const &AccountInfo, ACCOUNTS_LEN)
                .try_into()
                .unwrap() // this is safe as we know the length of the array
        };

        // Instruction data layout: Now uses only 3 proof offsets
        const DATA_LEN: usize = 1 + 1 + POD_AE_CIPHERTEXT_LEN + 1 + 1 + 1; // 41 bytes total
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];

        let balance_start = 2;
        let balance_end = balance_start + POD_AE_CIPHERTEXT_LEN; // 2 + 36 = 38
        let eq_offset_idx = balance_end; // 38
        let valid_offset_idx = eq_offset_idx + 1; // 39
        let range_offset_idx = valid_offset_idx + 1; // 40

        write_bytes(&mut instruction_data[0..1], &[27]); // Main discriminator
        write_bytes(&mut instruction_data[1..2], &[7]); // Transfer discriminator

        // New source decryptable balance
        write_bytes(
            &mut instruction_data[balance_start..balance_end],
            &self.new_source_decryptable_available_balance,
        );
        // Equality proof offset
        write_bytes(
            &mut instruction_data[eq_offset_idx..eq_offset_idx + 1],
            &[self.equality_proof_instruction_offset as u8],
        );
        // Validity proof offset
        write_bytes(
            &mut instruction_data[valid_offset_idx..valid_offset_idx + 1],
            &[self.transfer_amount_ciphertext_validity_proof_instruction_offset as u8],
        );
        // Range proof offset (single)
        write_bytes(
            &mut instruction_data[range_offset_idx..range_offset_idx + 1],
            &[self.range_proof_instruction_offset as u8],
        );

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: acc_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, DATA_LEN) },
        };

        // Pass reference to the array for accounts
        invoke_signed(&instruction, &acc_infos, signers)
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
pub struct TransferWithFee<'a, const ACCOUNTS_LEN: usize> {
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
    pub new_source_decryptable_available_balance: [u8; POD_AE_CIPHERTEXT_LEN],
    /// The transfer amount encrypted under the auditor ElGamal public key (low bits).
    pub transfer_amount_auditor_ciphertext_lo: [u8; POD_ELGAMAL_CIPHERTEXT_LEN],
    /// The transfer amount encrypted under the auditor ElGamal public key (high bits).
    pub transfer_amount_auditor_ciphertext_hi: [u8; POD_ELGAMAL_CIPHERTEXT_LEN],
    /// The fee commitment encrypted under the auditor ElGamal public key.
    pub fee_commitment_auditor_ciphertext: [u8; POD_ELGAMAL_CIPHERTEXT_LEN],
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

impl<const ACCOUNTS_LEN: usize> TransferWithFee<'_, ACCOUNTS_LEN> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // source, destination, mint
        let base_accounts = 3;
        // Check if the Instructions sysvar is required.
        // According to the Token-2022 documentation for TransferWithFee:
        // "4. [] (Optional) Instructions sysvar if at least one of the
        //     zk_elgamal_proof instructions are included in the same
        //     transaction."
        // A non-zero offset indicates a proof instruction is included inline
        // in the same transaction and needs to be accessed via the sysvar.
        // If all offsets are 0, all proofs are provided via context state accounts,
        // and the sysvar is not needed for this CPI.
        let sysvar_account = if self.equality_proof_instruction_offset != 0
            || self.transfer_amount_ciphertext_validity_proof_instruction_offset != 0
            || self.fee_sigma_proof_instruction_offset != 0
            || self.fee_ciphertext_validity_proof_instruction_offset != 0
            || self.range_proof_instruction_offset != 0
        {
            self.sysvar_instructions_account.is_some() as usize
        } else {
            0
        };

        // Count optional proof accounts provided.
        // These can be context state accounts (if offset is 0) or record accounts
        // used in conjunction with inline proofs (if offset is non-zero).
        let proof_accounts = self.equality_proof_account.is_some() as usize
            + self
                .transfer_amount_ciphertext_validity_proof_account
                .is_some() as usize
            + self.fee_sigma_proof_account.is_some() as usize
            + self.fee_ciphertext_validity_proof_account.is_some() as usize
            + self.range_proof_account.is_some() as usize;

        // Calculate the total expected number of accounts based on provided fields.
        let total_accounts = base_accounts + sysvar_account + proof_accounts + 1 /* authority */ + self.multisig_signers.len();

        // Validate the provided ACCOUNTS_LEN matches the calculated required accounts.
        if total_accounts != ACCOUNTS_LEN {
            return Err(ProgramError::InvalidArgument);
        }

        // Initialize arrays for account metas and infos using MaybeUninit for const generics.
        // Account metas
        const UNINIT_ACC_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut account_metas = [UNINIT_ACC_META; ACCOUNTS_LEN];

        // Account infos
        const UNINIT_ACC_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::<&AccountInfo>::uninit();
        let mut accounts = [UNINIT_ACC_INFO; ACCOUNTS_LEN];

        // Fill in base accounts (Source, Destination, Mint).
        // Accounts 1-3 in the documentation.
        account_metas[0].write(AccountMeta::writable(self.source_token_account.key()));
        accounts[0].write(self.source_token_account);

        account_metas[1].write(AccountMeta::writable(self.destination_token_account.key()));
        accounts[1].write(self.destination_token_account);

        // Mint is writable because the processor modifies withheld fee amounts.
        account_metas[2].write(AccountMeta::writable(self.mint.key()));
        accounts[2].write(self.mint);

        let mut idx = base_accounts;

        // Add sysvar if needed (based on the check above).
        // Account 4 in the documentation.
        if sysvar_account == 1 {
            // If sysvar is required (proofs are inline) but not provided, this unwrap will panic.
            // The caller must ensure sysvar_instructions_account is Some if any offset is non-zero.
            let sysvar = self.sysvar_instructions_account.unwrap();
            account_metas[idx].write(AccountMeta::readonly(sysvar.key()));
            accounts[idx].write(sysvar);
            idx += 1;
        }

        // Add all provided proof accounts in the documented order.
        // Accounts 5-9 in the documentation.
        let proof_accounts = [
            self.equality_proof_account,
            self.transfer_amount_ciphertext_validity_proof_account,
            self.fee_sigma_proof_account,
            self.fee_ciphertext_validity_proof_account,
            self.range_proof_account,
        ];

        for proof in proof_accounts.into_iter().flatten() {
            account_metas[idx].write(AccountMeta::readonly(proof.key()));
            accounts[idx].write(proof);
            idx += 1;
        }

        // Add Authority account.
        // Account 10 in the documentation (single signer case).
        if self.multisig_signers.is_empty() {
            // If no multisig signers, authority is the direct signer.
            account_metas[idx].write(AccountMeta::readonly_signer(self.authority.key()));
        } else {
            // If multisig signers are present, authority is not a direct signer itself.
            // It becomes Account 10 in the multisig documentation section.
            account_metas[idx].write(AccountMeta::readonly(self.authority.key()));
        }
        accounts[idx].write(self.authority);
        idx += 1;

        // Add Multisig signer accounts, if any.
        // Accounts 11+ in the multisig documentation section.
        for (i, signer) in self.multisig_signers.iter().enumerate() {
            // Note: The index here starts from `idx` which is *after* the authority account.
            account_metas[idx + i].write(AccountMeta::readonly_signer(signer.key()));
            accounts[idx + i].write(signer);
        }

        // Convert the initialized parts of the MaybeUninit arrays to safe slices.
        let acc_metas = unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, ACCOUNTS_LEN)
        };

        let acc_infos: [&AccountInfo; ACCOUNTS_LEN] = unsafe {
            // This conversion is safe because we previously validated that ACCOUNTS_LEN
            // matches the total number of accounts we added.
            core::slice::from_raw_parts(accounts.as_ptr() as *const &AccountInfo, ACCOUNTS_LEN)
                .try_into()
                .unwrap() // Safe as we verified the length
        };

        // Construct the instruction data according to the TransferWithFeeInstructionData layout.
        // Total size: 1 (Extension Disc) + 1 (Instruction Disc) + 36 (AE Ciphertext) + 3 * 64 (ElGamal Ciphertexts) + 5 (Proof Offsets) = 235 bytes
        const ACTUAL_DATA_LEN: usize =
            1 + 1 + POD_AE_CIPHERTEXT_LEN + (3 * POD_ELGAMAL_CIPHERTEXT_LEN) + 5;
        let mut instruction_data = [UNINIT_BYTE; ACTUAL_DATA_LEN];

        // Offsets within the instruction data buffer.
        let balance_start = 2;
        let balance_end = balance_start + POD_AE_CIPHERTEXT_LEN;
        let transfer_lo_start = balance_end;
        let transfer_lo_end = transfer_lo_start + POD_ELGAMAL_CIPHERTEXT_LEN;
        let transfer_hi_start = transfer_lo_end;
        let transfer_hi_end = transfer_hi_start + POD_ELGAMAL_CIPHERTEXT_LEN;
        let fee_commit_start = transfer_hi_end;
        let fee_commit_end = fee_commit_start + POD_ELGAMAL_CIPHERTEXT_LEN;

        let eq_offset_idx = fee_commit_end;
        let valid_offset_idx = eq_offset_idx + 1;
        let fee_sigma_offset_idx = valid_offset_idx + 1;
        let fee_valid_offset_idx = fee_sigma_offset_idx + 1;
        let range_offset_idx = fee_valid_offset_idx + 1;

        // Write discriminators
        write_bytes(&mut instruction_data[0..1], &[27]); // ConfidentialTransfer Extension discriminator
        write_bytes(&mut instruction_data[1..2], &[16]); // TransferWithFee instruction discriminator

        // Write ciphertexts
        write_bytes(
            &mut instruction_data[balance_start..balance_end],
            &self.new_source_decryptable_available_balance,
        );
        write_bytes(
            &mut instruction_data[transfer_lo_start..transfer_lo_end],
            &self.transfer_amount_auditor_ciphertext_lo,
        );
        write_bytes(
            &mut instruction_data[transfer_hi_start..transfer_hi_end],
            &self.transfer_amount_auditor_ciphertext_hi,
        );
        write_bytes(
            &mut instruction_data[fee_commit_start..fee_commit_end],
            &self.fee_commitment_auditor_ciphertext,
        );

        // Write proof instruction offsets (as u8)
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

        // Create the final Instruction struct
        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: acc_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, ACTUAL_DATA_LEN) },
        };

        // Invoke the CPI
        invoke_signed(&instruction, &acc_infos, signers)
    }
}
