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
pub enum TransferHookInstruction {
    Initialize,
    Update,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TransferHook {
    /// Authority that can set the transfer hook program id
    authority: Pubkey,
    /// Program that authorizes the transfer
    program_id: Pubkey,
}

impl TransferHook {
    pub const AUTHORITY_START: usize = 170;

    pub const BASE_LEN: usize = core::mem::size_of::<TransferHook>();

    pub const LEN: usize = Self::AUTHORITY_START + Self::BASE_LEN;

    /// Return a `TransferHook` from the given bytes (unsafe, unchecked).
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes[Self::AUTHORITY_START as usize..].as_ptr() as *const TransferHook)
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

    /// Creates a new state
    pub fn new(authority: Option<&Pubkey>, program_id: Option<&Pubkey>) -> Self {
        Self {
            authority: authority.map(|&x| x).unwrap_or_default(),
            program_id: program_id.map(|&x| x).unwrap_or_default(),
        }
    }
    
    #[inline(always)]
    pub fn has_authority(&self) -> bool {
        self.authority != Pubkey::default()
    }

    #[inline]
    pub fn authority(&self) -> Option<&Pubkey> {
        if self.has_authority() {
            Some(&self.authority)
        } else {
            None
        }
    }

    /// Return the authority.
    ///
    /// This method should be used when the caller knows that the transfer hook will have an
    /// authority set since it skips the `Option` check.
    #[inline(always)]
    pub fn authority_unchecked(&self) -> &Pubkey {
        &self.authority
    }

    #[inline(always)]
    pub fn has_program_id(&self) -> bool {
        self.program_id != Pubkey::default()
    }

    #[inline]
    pub fn program_id(&self) -> Option<&Pubkey> {
        if self.has_program_id() {
            Some(&self.program_id)
        } else {
            None
        }
    }

    /// Return the program id.
    ///
    /// This method should be used when the caller knows that the transfer hook will have a
    /// program id set since it skips the `Option` check.
    #[inline(always)]
    pub fn program_id_unchecked(&self) -> &Pubkey {
        &self.program_id
    }
}

pub fn transfer_hook_initialize_instruction_data<'a>(
    buffer: &'a mut [u8],
    instruction_type: TransferHookInstruction,
    authority: Option<&'a Pubkey>,
    program_id: Option<&'a Pubkey>,
) -> &'a [u8] {
    // instruction data
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    // -  [2..34]: authority (32 bytes, Pubkey)
    // -  [34..66]: program_id (32 bytes, Pubkey)
    
    // Set discriminators
    buffer[..2].copy_from_slice(&[
        ExtensionDiscriminator::TransferHook as u8,
        instruction_type as u8,
    ]);
    
    // Set authority at offset [2..34]
    if let Some(x) = authority {
        buffer[2..34].copy_from_slice(x);
    } else {
        buffer[2..34].copy_from_slice(&[0; 32]);
    }
    
    // Set program_id at offset [34..66]
    if let Some(x) = program_id {
        buffer[34..66].copy_from_slice(x);
    } else {
        buffer[34..66].copy_from_slice(&[0; 32]);
    }

    buffer
}

