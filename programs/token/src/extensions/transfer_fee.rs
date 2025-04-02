use core::{mem::MaybeUninit, slice::from_raw_parts};

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

use super::{get_extension_from_bytes, Extension};

/// Transfer fee configuration
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransferFee {
    /// First epoch where the transfer fee takes effect
    pub epoch: [u8; 8],
    /// Maximum fee assessed on transfers, expressed as an amount of tokens
    pub maximum_fee: [u8; 8],
    /// Amount of transfer collected as fees, expressed as basis points of the
    /// transfer amount, ie. increments of 0.01%
    pub transfer_fee_basis_points: [u8; 2],
}

/// State of the transfer fee configuration
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransferFeeConfig {
    /// Optional authority to set the fee
    pub transfer_fee_config_authority: Pubkey,
    /// Withdraw from mint instructions must be signed by this key
    pub withdraw_withheld_authority: Pubkey,
    /// Withheld transfer fee tokens that have been moved to the mint for
    /// withdrawal
    pub withheld_amount: [u8; 8],
    /// Older transfer fee, used if the current epoch < new_transfer_fee.epoch
    pub older_transfer_fee: TransferFee,
    /// Newer transfer fee, used if the current epoch >= new_transfer_fee.epoch
    pub newer_transfer_fee: TransferFee,
}

impl Extension for TransferFeeConfig {
    const TYPE: super::ExtensionType = super::ExtensionType::TransferFeeConfig;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl TransferFeeConfig {
    /// The length of the `TransferFeeConfig` account data.
    pub const LEN: usize = core::mem::size_of::<TransferFeeConfig>();

    /// Return a `TransferFeeConfig` from the given Mint account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<TransferFeeConfig, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let acc_data_bytes = account_info.try_borrow_data()?;
        let acc_data_bytes = acc_data_bytes.as_ref();

        get_extension_from_bytes::<Self>(acc_data_bytes).ok_or(ProgramError::InvalidAccountData)
    }
}

// Instructions
/// Initialize the transfer fee configuration for a mint.
pub struct InitializeTransferFeeConfig<'a> {
    // Mint account
    pub mint: &'a AccountInfo,
    /// Pubkey that may update the fees
    pub transfer_fee_config_authority: Option<Pubkey>,
    /// Withdraw instructions must be signed by this key
    pub withdraw_withheld_authority: Option<Pubkey>,
    /// Amount of transfer collected as fees, expressed as basis points of
    /// the transfer amount
    pub transfer_fee_basis_points: u16,
    /// Maximum fee assessed on transfers
    pub maximum_fee: u64,
}

impl InitializeTransferFeeConfig<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..2]: extension instruction discriminator (1 byte, u8)
        // -  [2..34]: mint (32 bytes, Pubkey)
        // -  [34..38]: transfer_fee_config_authority_flag (4 byte, [u8;4])
        // -  [38..70]: transfer_fee_config_authority (32 bytes, Pubkey)
        // -  [70..74]: withdraw_withheld_authority_flag (4 byte, [u8;4])
        // -  [74..106]: withdraw_withheld_authority (32 bytes, Pubkey)
        // -  [106..108]: transfer_fee_basis_points (2 bytes, u16)
        // -  [108..116]: maximum_fee (8 bytes, u64)

        let mut instruction_data = [UNINIT_BYTE; 116];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[26]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[0]);
        // Set mint as Pubkey at offset [1..33]
        write_bytes(&mut instruction_data[2..34], self.mint.key().as_ref());
        // Set transfer_fee_config_authority COption at offset [33..37]
        if let Some(transfer_fee_config_authority) = self.transfer_fee_config_authority {
            write_bytes(&mut instruction_data[34..38], &[1, 0, 0, 0]);
            write_bytes(
                &mut instruction_data[38..70],
                transfer_fee_config_authority.as_ref(),
            );
        } else {
            write_bytes(&mut instruction_data[34..38], &[0, 0, 0, 0]);
            write_bytes(&mut instruction_data[38..70], &Pubkey::default());
        }

        if let Some(withdraw_withheld_authority) = self.withdraw_withheld_authority {
            write_bytes(&mut instruction_data[70..74], &[1, 0, 0, 0]);
            write_bytes(
                &mut instruction_data[74..106],
                withdraw_withheld_authority.as_ref(),
            );
        } else {
            write_bytes(&mut instruction_data[70..74], &[0, 0, 0, 0]);
            write_bytes(&mut instruction_data[74..106], &Pubkey::default());
        }

        // Set transfer_fee_basis_points as u16 at offset [106..108]
        write_bytes(
            &mut instruction_data[106..108],
            &self.transfer_fee_basis_points.to_le_bytes(),
        );
        // Set maximum_fee as u64 at offset [108..116]
        write_bytes(
            &mut instruction_data[108..116],
            &self.maximum_fee.to_le_bytes(),
        );

        let instruction = Instruction {
            program_id: &crate::TOKEN_2022_PROGRAM_ID,
            accounts: &[AccountMeta::writable(self.mint.key())],
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 116) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}

/// Transfer tokens from one account to another, with a fee.
pub struct TransferCheckedWithFee<'a> {
    /// Source account
    pub source: &'a AccountInfo,
    /// Token mint
    pub mint: &'a AccountInfo,
    /// Destination account
    pub destination: &'a AccountInfo,
    /// Transfer authority (owner or delegate)
    pub authority: &'a AccountInfo,
    /// The amount of tokens to transfer.
    pub amount: u64,
    /// Expected number of base 10 digits to the right of the decimal place.
    pub decimals: u8,
    /// Expected fee assessed on this transfer, calculated off-chain based
    /// on the transfer_fee_basis_points and maximum_fee of the mint. May
    /// be 0 for a mint without a configured transfer fee.
    pub fee: u64,
}

impl TransferCheckedWithFee<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.source.key()),
            AccountMeta::writable(self.mint.key()),
            AccountMeta::writable(self.destination.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [2..10]: amount (8 bytes, u64)
        // -  [10]: decimals (1 byte, u8)
        // -  [11..19]: fee (8 bytes, u64)
        let mut instruction_data = [UNINIT_BYTE; 19];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[26]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[1]);
        // Set amount as u64 at offset [2..10]
        write_bytes(&mut instruction_data[2..10], &self.amount.to_le_bytes());
        // Set decimals as u8 at offset [10]
        write_bytes(&mut instruction_data[10..11], &[self.decimals]);
        // Set fee as u64 at offset [11..19]
        write_bytes(&mut instruction_data[11..19], &self.fee.to_le_bytes());

        let instruction = Instruction {
            program_id: &crate::TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 19) },
        };

        invoke_signed(
            &instruction,
            &[self.source, self.mint, self.destination, self.authority],
            signers,
        )
    }
}

/// Withdraw withheld tokens from the mint account.
pub struct WithdrawWithheldTokensFromMint<'a> {
    /// Mint account (must include the `TransferFeeConfig` extension)
    pub mint: &'a AccountInfo,
    /// The fee receiver account (must include the `TransferFeeAmount` extension associated with the provided mint)
    pub fee_receiver: &'a AccountInfo,
    /// The mint's `withdraw_withheld_authority`.
    pub withraw_withheld_authority: &'a AccountInfo,
}

impl WithdrawWithheldTokensFromMint<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::writable(self.fee_receiver.key()),
            AccountMeta::readonly_signer(self.withraw_withheld_authority.key()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator
        // -  [1]: extension instruction discriminator

        let instruction = Instruction {
            program_id: &crate::TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: &[26, 2],
        };

        invoke_signed(
            &instruction,
            &[
                self.mint,
                self.fee_receiver,
                self.withraw_withheld_authority,
            ],
            signers,
        )
    }
}

/// Withdraw withheld tokens from the provided source accounts.
pub struct WithdrawWithheldTokensFromAccounts<'a, const ACCOUNTS_LEN: usize> {
    /// Mint account (must include the `TransferFeeConfig` extension)
    pub mint: &'a AccountInfo,
    /// The fee receiver account (must include the `TransferFeeAmount` extension associated with the provided mint)
    pub fee_receiver: &'a AccountInfo,
    /// The mint's `withdraw_withheld_authority`.
    pub withdraw_withheld_authority: &'a AccountInfo,
    /// The source accounts to withdraw from.
    pub source_accounts: &'a [&'a AccountInfo],
}

impl<const ACCOUNTS_LEN: usize> WithdrawWithheldTokensFromAccounts<'_, ACCOUNTS_LEN> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        if 3 + self.source_accounts.len() != ACCOUNTS_LEN {
            return Err(ProgramError::Custom(1));
        }

        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if 3 + self.source_accounts.len() != ACCOUNTS_LEN {
            return Err(ProgramError::Custom(1));
        }
        // Account metads
        const UNINIT_ACC_METAS: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut account_metas = [UNINIT_ACC_METAS; ACCOUNTS_LEN];

        account_metas[0].write(AccountMeta::writable(self.mint.key()));
        account_metas[1].write(AccountMeta::writable(self.fee_receiver.key()));
        account_metas[2].write(AccountMeta::readonly_signer(
            self.withdraw_withheld_authority.key(),
        ));

        for (i, account) in self.source_accounts.iter().enumerate() {
            account_metas[3 + i].write(AccountMeta::writable(account.key()));
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator
        // -  [1]: extension instruction discriminator

        let acc_metas = unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, ACCOUNTS_LEN)
        };

        let instruction = Instruction {
            program_id: &crate::TOKEN_2022_PROGRAM_ID,
            accounts: acc_metas,
            data: &[26, 3],
        };

        const UNINIT_ACC_INFOS: MaybeUninit<&AccountInfo> = MaybeUninit::<&AccountInfo>::uninit();

        let mut accounts = [UNINIT_ACC_INFOS; ACCOUNTS_LEN];

        accounts[0].write(self.mint);
        accounts[1].write(self.fee_receiver);
        accounts[2].write(self.withdraw_withheld_authority);

        for (i, account) in self.source_accounts.iter().enumerate() {
            accounts[3 + i].write(account);
        }

        let acc_infos: [&AccountInfo; ACCOUNTS_LEN] = unsafe {
            core::slice::from_raw_parts(accounts.as_ptr() as *const &AccountInfo, ACCOUNTS_LEN)
                .try_into()
                .unwrap() // this is safe as we know the length of the array
        };

        invoke_signed(&instruction, &acc_infos, signers)
    }
}

/// Harvest withheld tokens to mint accounts.
pub struct HarvestWithheldTokensToMint<'a, const ACCOUNTS_LEN: usize> {
    /// Mint account (must include the `TransferFeeConfig` extension)
    mint: &'a AccountInfo,
    /// The source accounts to harvest from.
    source_accounts: &'a [&'a AccountInfo],
}

impl<const ACCOUNTS_LEN: usize> HarvestWithheldTokensToMint<'_, ACCOUNTS_LEN> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        if 1 + self.source_accounts.len() != ACCOUNTS_LEN {
            return Err(ProgramError::Custom(1));
        }
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if 1 + self.source_accounts.len() != ACCOUNTS_LEN {
            return Err(ProgramError::Custom(1));
        }

        // Account metads
        const UNINIT_ACC_METAS: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut account_metas = [UNINIT_ACC_METAS; ACCOUNTS_LEN];

        account_metas[0].write(AccountMeta::writable(self.mint.key()));

        for (i, account) in self.source_accounts.iter().enumerate() {
            account_metas[1 + i].write(AccountMeta::writable(account.key()));
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator
        // -  [1]: extension instruction discriminator

        let acc_metas = unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, ACCOUNTS_LEN)
        };

        let instruction = Instruction {
            program_id: &crate::TOKEN_2022_PROGRAM_ID,
            accounts: acc_metas,
            data: &[26, 4],
        };

        const UNINIT_ACC_INFOS: MaybeUninit<&AccountInfo> = MaybeUninit::<&AccountInfo>::uninit();

        let mut accounts = [UNINIT_ACC_INFOS; ACCOUNTS_LEN];

        accounts[0].write(self.mint);

        for (i, account) in self.source_accounts.iter().enumerate() {
            accounts[1 + i].write(account);
        }

        let acc_infos: [&AccountInfo; ACCOUNTS_LEN] = unsafe {
            core::slice::from_raw_parts(accounts.as_ptr() as *const &AccountInfo, ACCOUNTS_LEN)
                .try_into()
                .unwrap() // this is safe as we know the length of the array
        };

        invoke_signed(&instruction, &acc_infos, signers)
    }
}

/// Set the transfer fee configuration for a mint.
pub struct SetTransferFee<'a> {
    /// Mint account
    pub mint: &'a AccountInfo,
    /// The mint's fee account owner.
    pub mint_fee_acc_owner: &'a AccountInfo,
    /// Amount of transfer collected as fees, expressed as basis points of
    /// the transfer amount
    pub transfer_fee_basis_points: u16,
    /// Maximum fee assessed on transfers
    pub maximum_fee: u64,
}

impl SetTransferFee<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly(self.mint_fee_acc_owner.key()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [2..4]: transfer_fee_basis_points (2 bytes, u16)
        // -  [4..12]: maximum_fee (8 bytes, u64)
        let mut instruction_data = [UNINIT_BYTE; 12];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[26]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[5]);
        // Set transfer_fee_basis_points as u16 at offset [2..4]
        write_bytes(
            &mut instruction_data[2..4],
            &self.transfer_fee_basis_points.to_le_bytes(),
        );
        // Set maximum_fee as u64 at offset [3..12]
        write_bytes(
            &mut instruction_data[4..12],
            &self.maximum_fee.to_le_bytes(),
        );

        let instruction = Instruction {
            program_id: &crate::TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 12) },
        };

        invoke_signed(&instruction, &[self.mint, self.mint_fee_acc_owner], signers)
    }
}
