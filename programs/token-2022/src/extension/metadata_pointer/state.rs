use {
    crate::ID,
    pinocchio::{
        account_info::{AccountInfo, Ref},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

#[repr(u8)]
pub enum InstructionDiscriminatorMetadataPointer {
    Initialize = 0,
    Update = 1,
}

/// Instruction data layout:
/// - [0]                        : Extension discriminator (1 byte)
/// - [1]                        : Instruction discriminator (1 byte)
/// - [2..34]                    : authority pubkey (32 bytes)
/// - [34..66]                   : metadata_address pubkey (32 bytes)
pub mod offset_metadata_pointer_initialize {
    pub const START: u8 = 2;
    pub const AUTHORITY_PUBKEY: u8 = 32;
    pub const METADATA_ADDRESS_PUBKEY: u8 = 32;
    pub const END: u8 = START + AUTHORITY_PUBKEY + METADATA_ADDRESS_PUBKEY;
}

/// Instruction data layout:
/// - [0]                        : Extension discriminator (1 byte)
/// - [1]                        : Instruction discriminator (1 byte)
/// - [2..34]                    : metadata_address pubkey (32 bytes)
pub mod offset_metadata_pointer_update {
    pub const START: u8 = 2;
    pub const METADATA_ADDRESS_PUBKEY: u8 = 32;
    pub const END: u8 = START + METADATA_ADDRESS_PUBKEY;
}

/// Metadata pointer extension data for mints.
#[repr(C)]
pub struct MetadataPointer {
    /// Authority that can set the metadata address
    pub authority: Pubkey,
    /// Account address that holds the metadata
    pub metadata_address: Pubkey,
}

impl MetadataPointer {
    /// The length of the mint with `MetadataPointer` extension data
    const LEN: u8 = 234;

    /// The index where authority address starts in the mint with `MetadataPointer` extension data
    const AUTHORITY_START: u8 = 170;

    /// The length of the `MetadataPointer` extension data.
    pub const BASE_LEN: usize = core::mem::size_of::<MetadataPointer>();

    /// Return a `MetadataPointer` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<Ref<MetadataPointer>, ProgramError> {
        // Check data length first
        if account_info.data_len() < Self::LEN as usize {
            Err(ProgramError::InvalidAccountData)?;
        }

        // Check owner
        if account_info.owner() != &ID {
            Err(ProgramError::InvalidAccountOwner)?;
        }

        // Safely borrow and map the data
        let data_ref = account_info
            .try_borrow_data()
            .map_err(|_| ProgramError::AccountBorrowFailed)?;

        Ok(Ref::map(data_ref, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
    }

    /// Return a `MetadataPointer` from the given account info.
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

    /// Return a `MetadataPointer` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. `bytes` contains at least `LEN` bytes
    /// 2. `bytes` contains a valid representation of `MetadataPointer`
    /// 3. The data is properly aligned (though MetadataPointer has alignment of 1)
    /// 4. The bytes represent valid flag values and pubkey data
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes[Self::AUTHORITY_START as usize..].as_ptr() as *const MetadataPointer)
    }

    /// Safe version of from_bytes that performs validation
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() < Self::LEN as usize {
            Err(ProgramError::InvalidAccountData)?;
        }

        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Creates a new state
    pub fn new(authority: Option<&Pubkey>, metadata_address: Option<&Pubkey>) -> Self {
        Self {
            authority: authority.map(|&x| x).unwrap_or_default(),
            metadata_address: metadata_address.map(|&x| x).unwrap_or_default(),
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
    /// This method should be used when the caller knows that the Metadata pointer will have an
    /// authority set since it skips the `Option` check.
    #[inline(always)]
    pub fn authority_unchecked(&self) -> &Pubkey {
        &self.authority
    }

    #[inline(always)]
    pub fn has_metadata_address(&self) -> bool {
        self.metadata_address != Pubkey::default()
    }

    #[inline]
    pub fn metadata_address(&self) -> Option<&Pubkey> {
        if self.has_metadata_address() {
            Some(&self.metadata_address)
        } else {
            None
        }
    }

    /// Return the Metadata address.
    ///
    /// This method should be used when the caller knows that the metadata pointer will have a
    /// metadata address set since it skips the `Option` check.
    #[inline(always)]
    pub fn metadata_address_unchecked(&self) -> &Pubkey {
        &self.metadata_address
    }
}
