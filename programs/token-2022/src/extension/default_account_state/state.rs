use core::mem::MaybeUninit;
use crate::{write_bytes, UNINIT_BYTE, ID};
use crate::extension::consts::ExtensionDiscriminator;
use pinocchio::{
    account_info::{AccountInfo, Ref},
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Default Account State extension instructions
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum DefaultAccountStateInstruction {
    /// Initialize a new mint with the default state for new Accounts
    Initialize = 0,
    /// Update the default state for new Accounts
    Update = 1,
}

/// Configuration for the DefaultAccountState extension
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DefaultAccountStateConfig {
    /// Default state for new accounts
    pub state: u8,
}

impl DefaultAccountStateConfig {
    pub const AUTHORITY_START: usize = 170;

    pub const BASE_LEN: usize = core::mem::size_of::<DefaultAccountStateConfig>();

    pub const LEN: usize = Self::AUTHORITY_START + Self::BASE_LEN;

    /// Return a `DefaultAccountStateConfig` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<Self, pinocchio::program_error::ProgramError> {
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

    /// Return a `DefaultAccountStateConfig` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, but does not
    /// perform the borrow check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to borrow the account data (e.g., there are
    /// no mutable borrows of the account data).
    #[inline]
    pub unsafe fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&Self, ProgramError> {
        // Check data length first
        if account_info.data_len() < Self::LEN as usize {
            Err(ProgramError::InvalidAccountData)?;
        }

        // Check owner
        if account_info.owner() != &ID {
            Err(ProgramError::InvalidAccountOwner)?;
        }

        // Get unchecked borrow and convert
        let data = account_info.borrow_data_unchecked();
        Ok(Self::from_bytes_unchecked(data))
    }

    /// Return a `DefaultAccountStateConfig` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. `bytes` contains at least `LEN` bytes
    /// 2. `bytes` contains a valid representation of `DefaultAccountStateConfig`
    /// 3. The data is properly aligned
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes[Self::AUTHORITY_START as usize..].as_ptr() as *const DefaultAccountStateConfig)
    }

    /// Safe version of from_bytes that performs validation
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() < Self::LEN as usize {
            Err(ProgramError::InvalidAccountData)?;
        }

        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Get the default account state
    #[inline(always)]
    pub fn state(&self) -> u8 {
        self.state
    }
}

pub fn default_account_state_instruction_data(
    instruction_type: DefaultAccountStateInstruction,
    state: u8,
) -> [MaybeUninit<u8>; 3] {
    // instruction data
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    // -  [2]: state (1 byte, u8)
    
    const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::uninit();
    let mut data = [UNINIT_BYTE; 3];
    // Set extension discriminator at offset [0]
    write_bytes(&mut data, &[ExtensionDiscriminator::DefaultAccountState as u8]);
    // Set sub-instruction at offset [1]
    write_bytes(&mut data[1..2], &[instruction_type as u8]);
    // Set state at offset [2]
    write_bytes(&mut data[2..3], &[state]);

    data
}
