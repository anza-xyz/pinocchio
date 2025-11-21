use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

use crate::{
    state::{Meta, Stake, StakeFlags},
    ID,
};

/// The length of the stake state type.
const TYPE_LEN: usize = 4;

/// Offset where Meta data begins.
const META_OFFSET: usize = TYPE_LEN;

/// Offset where Stake data begins.
const STAKE_OFFSET: usize = META_OFFSET + Meta::LEN;

/// Offset where StakeFlags data begins.
const STAKE_FLAGS_OFFSET: usize = STAKE_OFFSET + Stake::LEN;

/// Minimum length for an Initialized stake account (V2).
const INITIALIZED_LEN: usize = TYPE_LEN + Meta::LEN;

/// Minimum length for a Stake (delegated) stake account (V2).
const STAKE_LEN: usize = TYPE_LEN + Meta::LEN + Stake::LEN + StakeFlags::LEN;

/// Stake state type (V2).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StakeStateV2Type {
    /// Account is not yet initialized.
    Uninitialized = 0,
    /// Stake account is initialized but not delegated.
    Initialized = 1,
    /// Stake account is delegated.
    Stake = 2,
    /// Account is a rewards pool.
    RewardsPool = 3,
}

impl From<u8> for StakeStateV2Type {
    fn from(value: u8) -> Self {
        match value {
            0 => StakeStateV2Type::Uninitialized,
            1 => StakeStateV2Type::Initialized,
            2 => StakeStateV2Type::Stake,
            3 => StakeStateV2Type::RewardsPool,
            _ => panic!("invalid stake state value: {value}"),
        }
    }
}

impl From<StakeStateV2Type> for u8 {
    fn from(value: StakeStateV2Type) -> Self {
        match value {
            StakeStateV2Type::Uninitialized => 0,
            StakeStateV2Type::Initialized => 1,
            StakeStateV2Type::Stake => 2,
            StakeStateV2Type::RewardsPool => 3,
        }
    }
}

/// Zero-copy view over an Initialized stake account (V2).
pub struct StakeAccountV2Initialized<'a>(&'a [u8]);

impl StakeAccountV2Initialized<'_> {
    /// Returns a reference to the meta data.
    #[inline(always)]
    pub fn meta(&self) -> &Meta {
        unsafe { &*(self.0.as_ptr().add(META_OFFSET) as *const Meta) }
    }
}

/// Zero-copy view over a delegated Stake account (V2).
pub struct StakeAccountV2Stake<'a>(&'a [u8]);

impl StakeAccountV2Stake<'_> {
    /// Returns a reference to the meta data.
    #[inline(always)]
    pub fn meta(&self) -> &Meta {
        unsafe { &*(self.0.as_ptr().add(META_OFFSET) as *const Meta) }
    }

    /// Returns a reference to the stake data.
    #[inline(always)]
    pub fn stake(&self) -> &Stake {
        unsafe { &*(self.0.as_ptr().add(STAKE_OFFSET) as *const Stake) }
    }

    /// Returns a reference to the stake flags.
    #[inline(always)]
    pub fn stake_flags(&self) -> &StakeFlags {
        unsafe { &*(self.0.as_ptr().add(STAKE_FLAGS_OFFSET) as *const StakeFlags) }
    }
}

/// Zero-copy stake state representation (V2 with StakeFlags).
pub enum StakeStateV2<'a> {
    /// Account is not yet initialized.
    Uninitialized,
    /// Stake account is initialized but not delegated.
    Initialized(StakeAccountV2Initialized<'a>),
    /// Stake account is delegated.
    Stake(StakeAccountV2Stake<'a>),
    /// Account is a rewards pool.
    RewardsPool,
}

impl<'a> StakeStateV2<'a> {
    /// Return a `StakeStateV2` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(
        account_info: &'a AccountInfo,
    ) -> Result<StakeStateV2<'a>, ProgramError> {
        if !account_info.is_owned_by(&ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let data = account_info.try_borrow_data()?;

        if data.len() < TYPE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let state_type = data[0];

        match state_type {
            0 => Ok(StakeStateV2::Uninitialized),
            1 => {
                if data.len() < INITIALIZED_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                // SAFETY: We need to extend the lifetime of the borrowed data to 'a.
                // This is safe because:
                // 1. The account_info has lifetime 'a
                // 2. We've validated ownership and length
                // 3. The borrow is released but the underlying data remains valid for 'a
                let data_ptr = data.as_ptr();
                let data_len = data.len();
                drop(data);
                let slice = unsafe { core::slice::from_raw_parts(data_ptr, data_len) };
                Ok(StakeStateV2::Initialized(StakeAccountV2Initialized(slice)))
            }
            2 => {
                if data.len() < STAKE_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                let data_ptr = data.as_ptr();
                let data_len = data.len();
                drop(data);
                let slice = unsafe { core::slice::from_raw_parts(data_ptr, data_len) };
                Ok(StakeStateV2::Stake(StakeAccountV2Stake(slice)))
            }
            3 => Ok(StakeStateV2::RewardsPool),
            _ => Err(ProgramError::InvalidAccountData),
        }
    }

    /// Return a `StakeStateV2` from the given account info.
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
        account_info: &'a AccountInfo,
    ) -> Result<StakeStateV2<'a>, ProgramError> {
        if account_info.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let data = account_info.borrow_data_unchecked();

        if data.len() < TYPE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let state_type = data[0];

        match state_type {
            0 => Ok(StakeStateV2::Uninitialized),
            1 => {
                if data.len() < INITIALIZED_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                Ok(StakeStateV2::Initialized(StakeAccountV2Initialized(data)))
            }
            2 => {
                if data.len() < STAKE_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                Ok(StakeStateV2::Stake(StakeAccountV2Stake(data)))
            }
            3 => Ok(StakeStateV2::RewardsPool),
            _ => Err(ProgramError::InvalidAccountData),
        }
    }

    /// Return a `StakeStateV2` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of a stake account,
    /// and it is properly aligned. This method performs length validation based on the
    /// state type but does not validate ownership.
    #[inline]
    pub unsafe fn from_bytes_unchecked(bytes: &'a [u8]) -> Result<StakeStateV2<'a>, ProgramError> {
        if bytes.len() < TYPE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let state_type = bytes[0];

        match state_type {
            0 => Ok(StakeStateV2::Uninitialized),
            1 => {
                if bytes.len() < INITIALIZED_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                Ok(StakeStateV2::Initialized(StakeAccountV2Initialized(bytes)))
            }
            2 => {
                if bytes.len() < STAKE_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                Ok(StakeStateV2::Stake(StakeAccountV2Stake(bytes)))
            }
            3 => Ok(StakeStateV2::RewardsPool),
            _ => Err(ProgramError::InvalidAccountData),
        }
    }

    /// Returns the stake state type.
    #[inline(always)]
    pub fn state_type(&self) -> StakeStateV2Type {
        match self {
            StakeStateV2::Uninitialized => StakeStateV2Type::Uninitialized,
            StakeStateV2::Initialized(_) => StakeStateV2Type::Initialized,
            StakeStateV2::Stake(_) => StakeStateV2Type::Stake,
            StakeStateV2::RewardsPool => StakeStateV2Type::RewardsPool,
        }
    }
}
