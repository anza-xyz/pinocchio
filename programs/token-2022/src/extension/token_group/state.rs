use {
    crate::ID,
    core::mem,
    pinocchio::{
        account_info::{AccountInfo, Ref},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

#[repr(u64)]
pub enum InstructionDiscriminatorTokenGroup {
    InitializeGroup = 288286683834380665, // [121, 113, 108, 39, 54, 51, 0, 4]
    UpdateGroupMaxSize = 7931435946663945580, // [108, 37, 171, 143, 248, 30, 18, 110]
    UpdateGroupAuthority = 14688734194668431777, // [161, 105, 88, 1, 237, 221, 216, 203]
    InitializeMember = 9688630243381616792, // [152, 32, 222, 176, 223, 237, 116, 134]
}

/// Instruction data layout:
/// - [0..8]                     : Instruction discriminator (8 bytes)
/// - [8..40]                    : update_authority pubkey (32 bytes)
/// - [40..48]                   : max_size (8 bytes)
pub mod offset_token_group_initialize_group {
    pub const START: u8 = 8;
    pub const UPDATE_AUTHORITY: u8 = 32;
    pub const MAX_SIZE: u8 = 8;
    pub const END: u8 = START + UPDATE_AUTHORITY + MAX_SIZE;
}

/// Instruction data layout:
/// - [0..8]                     : Instruction discriminator (8 bytes)
/// - [8..16]                    : max_size (8 bytes)
pub mod offset_token_group_update_max_size {
    pub const START: u8 = 8;
    pub const MAX_SIZE: u8 = 8;
    pub const END: u8 = START + MAX_SIZE;
}

/// Instruction data layout:
/// - [0..8]                     : Instruction discriminator (8 bytes)
/// - [8..40]                    : new_authority pubkey (32 bytes)
pub mod offset_token_group_update_authority {
    pub const START: u8 = 8;
    pub const NEW_AUTHORITY: u8 = 32;
    pub const END: u8 = START + NEW_AUTHORITY;
}

/// Instruction data layout:
/// - [0..8]                     : Instruction discriminator (8 bytes)
pub mod offset_token_group_initialize_member {
    pub const START: u8 = 8;
    pub const END: u8 = START;
}

/// Data struct for a `TokenGroup`
#[repr(C)]
pub struct TokenGroup {
    /// The authority that can sign to update the group
    update_authority: Pubkey,
    /// The associated mint, used to counter spoofing to be sure that group
    /// belongs to a particular mint
    mint: Pubkey,
    /// The current number of group members
    size: u64,
    /// The maximum number of group members
    max_size: u64,
}

impl TokenGroup {
    /// The length of the account with `TokenGroup` data
    const LEN: usize = Self::DATA_START + Self::BASE_LEN;

    /// The index where TokenGroup data starts
    const DATA_START: usize = 238;

    /// The length of the `TokenGroup` data.
    pub const BASE_LEN: usize = mem::size_of::<TokenGroup>();

    /// Return a `TokenGroup` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<Ref<TokenGroup>, ProgramError> {
        // Check data length first
        if account_info.data_len() < Self::LEN {
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

    /// Return a `TokenGroup` from the given account info.
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
        if account_info.data_len() < Self::LEN {
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

    /// Return a `TokenGroup` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. `bytes` contains at least `LEN` bytes
    /// 2. `bytes` contains a valid representation of `TokenGroup`
    /// 3. The data is properly aligned
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes[Self::DATA_START..].as_ptr() as *const TokenGroup)
    }

    /// Safe version of from_bytes that performs validation
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() < Self::LEN {
            Err(ProgramError::InvalidAccountData)?;
        }

        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Creates a new `TokenGroup` state
    pub fn new(mint: &Pubkey, update_authority: Option<&Pubkey>, max_size: u64) -> Self {
        Self {
            mint: *mint,
            update_authority: update_authority.map(|&x| x).unwrap_or_default(),
            size: 0,
            max_size,
        }
    }

    #[inline(always)]
    pub fn has_update_authority(&self) -> bool {
        self.update_authority != Pubkey::default()
    }

    #[inline]
    pub fn update_authority(&self) -> Option<&Pubkey> {
        if self.has_update_authority() {
            Some(&self.update_authority)
        } else {
            None
        }
    }

    /// Return the update authority.
    ///
    /// This method should be used when the caller knows that the token group will have an
    /// update authority set since it skips the `Option` check.
    #[inline(always)]
    pub fn update_authority_unchecked(&self) -> &Pubkey {
        &self.update_authority
    }

    #[inline(always)]
    pub fn mint(&self) -> &Pubkey {
        &self.mint
    }

    #[inline(always)]
    pub fn size(&self) -> u64 {
        self.size
    }

    #[inline(always)]
    pub fn max_size(&self) -> u64 {
        self.max_size
    }

    /// Updates the max size for a group
    pub fn update_max_size(&mut self, new_max_size: u64) -> Result<(), ProgramError> {
        // The new max size cannot be less than the current size
        if new_max_size < self.size {
            Err(TokenGroupError::SizeExceedsNewMaxSize)?;
        }
        self.max_size = new_max_size;
        Ok(())
    }

    /// Increment the size for a group, returning the new size
    pub fn increment_size(&mut self) -> Result<u64, ProgramError> {
        // The new size cannot be greater than the max size
        let new_size = self
            .size
            .checked_add(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        if new_size > self.max_size {
            Err(TokenGroupError::SizeExceedsMaxSize)?;
        }
        self.size = new_size;
        Ok(new_size)
    }
}

/// Data struct for a `TokenGroupMember`
#[repr(C)]
pub struct TokenGroupMember {
    /// The associated mint, used to counter spoofing to be sure that member
    /// belongs to a particular mint
    mint: Pubkey,
    /// The pubkey of the `TokenGroup`
    group: Pubkey,
    /// The member number
    member_number: u64,
}

impl TokenGroupMember {
    /// The length of the account with `TokenGroupMember` data
    const LEN: usize = Self::DATA_START + Self::BASE_LEN;

    /// The index where TokenGroupMember data starts
    const DATA_START: usize = 238;

    /// The length of the `TokenGroupMember` data.
    pub const BASE_LEN: usize = mem::size_of::<TokenGroupMember>();

    /// Return a `TokenGroupMember` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<Ref<TokenGroupMember>, ProgramError> {
        // Check data length first
        if account_info.data_len() < Self::LEN {
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

    /// Return a `TokenGroupMember` from the given account info.
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
        if account_info.data_len() < Self::LEN {
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

    /// Return a `TokenGroupMember` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. `bytes` contains at least `LEN` bytes
    /// 2. `bytes` contains a valid representation of `TokenGroupMember`
    /// 3. The data is properly aligned
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes[Self::DATA_START..].as_ptr() as *const TokenGroupMember)
    }

    /// Safe version of from_bytes that performs validation
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() < Self::LEN {
            Err(ProgramError::InvalidAccountData)?;
        }

        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Creates a new `TokenGroupMember` state
    pub fn new(mint: &Pubkey, group: &Pubkey, member_number: u64) -> Self {
        Self {
            mint: *mint,
            group: *group,
            member_number,
        }
    }

    #[inline(always)]
    pub fn mint(&self) -> &Pubkey {
        &self.mint
    }

    #[inline(always)]
    pub fn group(&self) -> &Pubkey {
        &self.group
    }

    #[inline(always)]
    pub fn member_number(&self) -> u64 {
        self.member_number
    }
}

/// Errors that may be returned by the interface.
#[repr(u32)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenGroupError {
    /// Size is greater than proposed max size
    SizeExceedsNewMaxSize = 3_406_457_176,
    /// Size is greater than max size
    SizeExceedsMaxSize,
    /// Group is immutable
    ImmutableGroup,
    /// Incorrect mint authority has signed the instruction
    IncorrectMintAuthority,
    /// Incorrect update authority has signed the instruction
    IncorrectUpdateAuthority,
    /// Member account should not be the same as the group account
    MemberAccountIsGroupAccount,
}

impl From<TokenGroupError> for ProgramError {
    fn from(e: TokenGroupError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl pinocchio::program_error::ToStr for TokenGroupError {
    fn to_str<E>(&self) -> &'static str
    where
        E: 'static + pinocchio::program_error::ToStr + TryFrom<u32>,
    {
        match self {
            TokenGroupError::SizeExceedsNewMaxSize => "Size is greater than proposed max size",
            TokenGroupError::SizeExceedsMaxSize => "Size is greater than max size",
            TokenGroupError::ImmutableGroup => "Group is immutable",
            TokenGroupError::IncorrectMintAuthority => {
                "Incorrect mint authority has signed the instruction"
            }
            TokenGroupError::IncorrectUpdateAuthority => {
                "Incorrect update authority has signed the instruction"
            }
            TokenGroupError::MemberAccountIsGroupAccount => {
                "Member account should not be the same as the group account"
            }
        }
    }
}
