use core::mem::MaybeUninit;
use crate::{write_bytes, UNINIT_BYTE};
use crate::extension::consts::ExtensionDiscriminator;
use pinocchio::pubkey::Pubkey;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScaledUiAmountInstruction {
    Initialize,
    UpdateMultiplier,
}

/// Configuration for the ScaledUiAmount extension
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaledUiAmountConfig {
    /// Authority that can set the scaling amount and authority
    pub authority: Pubkey,
    /// Amount to multiply raw amounts by, outside of the decimal
    pub multiplier: f64,
    /// Unix timestamp at which `new_multiplier` comes into effective
    pub new_multiplier_effective_timestamp: i64,
    /// Next multiplier, once `new_multiplier_effective_timestamp` is reached
    pub new_multiplier: f64,
}

impl ScaledUiAmountConfig {    
    /// The index where authority address starts in the mint with `ScaledUiAmount` extension data
    pub const AUTHORITY_START: usize = 170;

    /// The length of the `ScaledUiAmount` extension data.
    pub const BASE_LEN: usize = core::mem::size_of::<ScaledUiAmountConfig>();

    /// The length of the mint with `ScaledUiAmount` extension data
    pub const LEN: usize = Self::AUTHORITY_START + Self::BASE_LEN;

    /// Return a `ScaledUiAmountConfig` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    pub fn from_account_info(account_info: &pinocchio::account_info::AccountInfo) -> Result<Self, pinocchio::program_error::ProgramError> {
        let data = account_info.try_borrow_data()?;
        if data.len() < Self::LEN as usize {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
        
        // SAFETY: We've validated the length above
        let config = unsafe {
            core::ptr::read((data.as_ptr().add(Self::AUTHORITY_START as usize)) as *const Self)
        };
        
        Ok(config)
    }
    
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes[Self::AUTHORITY_START as usize..].as_ptr() as *const ScaledUiAmountConfig)
    }

    /// Safe version of from_bytes that performs validation
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, pinocchio::program_error::ProgramError> {
        if bytes.len() < Self::LEN as usize {
            Err(pinocchio::program_error::ProgramError::InvalidAccountData)?;
        }

        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Get the authority
    #[inline]
    pub fn authority(&self) -> &Pubkey {
        &self.authority
    }

    /// Get the multiplier
    #[inline]
    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }
}

pub fn scaled_ui_amount_initialize_instruction_data(
    instruction_type: ScaledUiAmountInstruction,
    authority: Pubkey,
    multiplier: f64,
) -> [MaybeUninit<u8>; 42] {
    // instruction data
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    // -  [2..34]: authority (32 bytes, Pubkey)
    // -  [34..42]: multiplier (8 bytes, f64)
    
    let mut data = [UNINIT_BYTE; 42];
    // Set extension discriminator at offset [0]
    write_bytes(&mut data, &[ExtensionDiscriminator::ScaledUiAmount as u8]);
    // Set sub-instruction at offset [1]
    write_bytes(&mut data[1..2], &[instruction_type as u8]);
    // Set authority at offset [2..34] (32 bytes)
    write_bytes(&mut data[2..34], &authority);
    // Set multiplier at offset [34..42] (8 bytes)
    write_bytes(&mut data[34..42], &multiplier.to_le_bytes());

    data
}

pub fn scaled_ui_amount_update_multiplier_instruction_data(
    instruction_type: ScaledUiAmountInstruction,
    multiplier: f64,
    effective_timestamp: i64,
) -> [MaybeUninit<u8>; 18] {
    // instruction data
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    // -  [2..10]: multiplier (8 bytes, f64)
    // -  [10..18]: effective_timestamp (8 bytes, i64)
    
    let mut data = [UNINIT_BYTE; 18];
    // Set extension discriminator at offset [0]
    write_bytes(&mut data, &[ExtensionDiscriminator::ScaledUiAmount as u8]);
    // Set sub-instruction at offset [1]
    write_bytes(&mut data[1..2], &[instruction_type as u8]);
    // Set multiplier at offset [2..10]
    write_bytes(&mut data[2..10], &multiplier.to_le_bytes());
    // Set effective_timestamp at offset [10..18]
    write_bytes(&mut data[10..18], &effective_timestamp.to_le_bytes());

    data
}
