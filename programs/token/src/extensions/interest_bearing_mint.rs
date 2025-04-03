use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

use super::get_extension_from_bytes;

/// State for an interest-bearing token
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterestBearingConfig {
    /// Authority that can set the interest rate and authority
    pub rate_authority: Pubkey,
    /// Timestamp of initialization, from which to base interest calculations
    pub initialization_timestamp: [i8; 8],
    /// Average rate from initialization until the last time it was updated
    pub pre_update_average_rate: [u8; 2],
    /// Timestamp of the last update, used to calculate the total amount accrued
    pub last_update_timestamp: [i8; 8],
    /// Current rate, since the last update
    pub current_rate: [u8; 2],
}

impl super::Extension for InterestBearingConfig {
    const TYPE: super::ExtensionType = super::ExtensionType::InterestBearingConfig;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl InterestBearingConfig {
    /// The length of the `InterestBearingConfig` account data.
    pub const LEN: usize = core::mem::size_of::<InterestBearingConfig>();

    /// Return an `InterestBearingConfig` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<InterestBearingConfig, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let acc_data_bytes = account_info.try_borrow_data()?;
        let acc_data_bytes = acc_data_bytes.as_ref();

        get_extension_from_bytes::<Self>(acc_data_bytes).ok_or(ProgramError::InvalidAccountData)
    }
}

// Instructions
pub struct Initialize<'a> {
    /// The mint to initialize as interest-bearing
    pub mint: &'a AccountInfo,
    /// The public key for the account that can update the rate
    pub rate_authority: Pubkey,
    /// The initial interest rate
    pub rate: u16,
}

impl Initialize<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [AccountMeta::writable(self.mint.key())];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [2..4]: rate (2 bytes, u16)
        // -  [4..36]: rate authority (32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 36];
        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[33]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[0]);
        // Set rate as u16 at offset [1..3]
        write_bytes(&mut instruction_data[2..4], &self.rate.to_le_bytes());
        // Set rate authority as Pubkey at offset [3..35]
        write_bytes(&mut instruction_data[4..36], &self.rate_authority);

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 36) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}

pub struct Update<'a> {
    /// The mint to update interest rate
    pub mint: &'a AccountInfo,
    /// The mint rate authority
    pub rate_authority: &'a AccountInfo,
    /// The new interest rate
    pub new_rate: u16,
}

impl Update<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.rate_authority.key()),
        ];

        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: extension instruction discriminator (1 byte, u8)
        // -  [2..4]: new rate (2 bytes, u16)
        let mut instruction_data = [UNINIT_BYTE; 4];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[33]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[1]);
        // Set rate as u16 at offset [2..4]
        write_bytes(&mut instruction_data[2..4], &self.new_rate.to_le_bytes());

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 4) },
        };

        invoke_signed(&instruction, &[self.mint, self.rate_authority], signers)
    }
}
