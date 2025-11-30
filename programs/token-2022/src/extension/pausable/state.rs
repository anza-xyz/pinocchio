use core::mem::MaybeUninit;
use crate::{write_bytes, UNINIT_BYTE, ID};
use crate::extension::consts::ExtensionDiscriminator;
use pinocchio::{
    account_info::{AccountInfo, Ref},
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PausableInstruction {
    Initialize,
    Pause,
    Resume,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PausableConfig {
    /// raw authority bytes (32)
    authority: Pubkey,
    /// Whether token is paused
    paused: u8,
}

impl PausableConfig {
    pub const AUTHORITY_START: usize = 170;

    pub const BASE_LEN: usize = core::mem::size_of::<PausableConfig>();

    pub const LEN: usize = Self::AUTHORITY_START + Self::BASE_LEN;

    /// Return a `PausableConfig` from the given bytes (unsafe, unchecked).
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes[Self::AUTHORITY_START as usize..].as_ptr() as *const PausableConfig)
    }

    /// Safe version that validates lengths
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() < Self::LEN {
            Err(ProgramError::InvalidAccountData)?;
        }
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Like your old from_account_info method
    #[inline]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<Self, ProgramError> {
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
    
    /// Get the authority
    #[inline]
    pub fn authority(&self) -> &Pubkey {
        &self.authority
    }

    /// Check if the mint is paused
    #[inline(always)]
    pub fn is_paused(&self) -> bool {
        self.paused != 0
    }
}


pub fn pausable_instruction_data(
    instruction_type: PausableInstruction,
) -> [MaybeUninit<u8>; 2] {
    // instruction data
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    
    let mut data = [UNINIT_BYTE; 2];
    // Set extension discriminator at offset [0]
    write_bytes(&mut data, &[ExtensionDiscriminator::Pausable as u8]);
    // Set sub-instruction at offset [1]
    write_bytes(&mut data[1..2], &[instruction_type as u8]);

    data
}

pub fn pausable_initialize_instruction_data(
    instruction_type: PausableInstruction,
    authority: Pubkey,
) -> [MaybeUninit<u8>; 34] {
    // instruction data
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    // -  [2..34]: authority (32 bytes, Pubkey)
    
    let mut data = [UNINIT_BYTE; 34];
    // Set extension discriminator at offset [0]
    write_bytes(&mut data, &[ExtensionDiscriminator::Pausable as u8]);
    // Set sub-instruction at offset [1]
    write_bytes(&mut data[1..2], &[instruction_type as u8]);
    // Set authority at offset [2..34]
    write_bytes(&mut data[2..34], &authority);

    data
}
