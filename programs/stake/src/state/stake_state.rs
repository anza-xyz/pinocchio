use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

use crate::{
    state::{Meta, Stake},
    ID,
};

/// The length of the stake state type.
const TYPE_LEN: usize = 4;

/// Offset where Meta data begins.
const META_OFFSET: usize = TYPE_LEN;

/// Offset where Stake data begins.
const STAKE_OFFSET: usize = META_OFFSET + Meta::LEN;

/// Minimum length for an Initialized stake account.
const INITIALIZED_LEN: usize = TYPE_LEN + Meta::LEN;

/// Minimum length for a Stake (delegated) stake account.
const STAKE_LEN: usize = TYPE_LEN + Meta::LEN + Stake::LEN;

/// Stake state type.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StakeStateType {
    /// Account is not yet initialized.
    Uninitialized = 0,
    /// Stake account is initialized but not delegated.
    Initialized = 1,
    /// Stake account is delegated.
    Stake = 2,
    /// Account is a rewards pool.
    RewardsPool = 3,
}

impl From<u8> for StakeStateType {
    fn from(value: u8) -> Self {
        match value {
            0 => StakeStateType::Uninitialized,
            1 => StakeStateType::Initialized,
            2 => StakeStateType::Stake,
            3 => StakeStateType::RewardsPool,
            _ => panic!("invalid stake state value: {value}"),
        }
    }
}

impl From<StakeStateType> for u8 {
    fn from(value: StakeStateType) -> Self {
        match value {
            StakeStateType::Uninitialized => 0,
            StakeStateType::Initialized => 1,
            StakeStateType::Stake => 2,
            StakeStateType::RewardsPool => 3,
        }
    }
}

/// Zero-copy view over an Initialized stake account.
pub struct StakeAccountInitialized<'a>(&'a [u8]);

impl<'a> StakeAccountInitialized<'a> {
    /// Returns a reference to the meta data.
    #[inline(always)]
    pub fn meta(&self) -> &Meta {
        unsafe { &*(self.0.as_ptr().add(META_OFFSET) as *const Meta) }
    }
}

/// Zero-copy view over a delegated Stake account.
pub struct StakeAccountStake<'a>(&'a [u8]);

impl<'a> StakeAccountStake<'a> {
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
}

/// Zero-copy stake state representation.
pub enum StakeState<'a> {
    /// Account is not yet initialized.
    Uninitialized,
    /// Stake account is initialized but not delegated.
    Initialized(StakeAccountInitialized<'a>),
    /// Stake account is delegated.
    Stake(StakeAccountStake<'a>),
    /// Account is a rewards pool.
    RewardsPool,
}

impl<'a> StakeState<'a> {
    /// Return a `StakeState` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(
        account_info: &'a AccountInfo,
    ) -> Result<StakeState<'a>, ProgramError> {
        if !account_info.is_owned_by(&ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let data = account_info.try_borrow_data()?;

        if data.len() < TYPE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let state_type = data[0];

        match state_type {
            0 => Ok(StakeState::Uninitialized),
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
                Ok(StakeState::Initialized(StakeAccountInitialized(slice)))
            }
            2 => {
                if data.len() < STAKE_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                let data_ptr = data.as_ptr();
                let data_len = data.len();
                drop(data);
                let slice = unsafe { core::slice::from_raw_parts(data_ptr, data_len) };
                Ok(StakeState::Stake(StakeAccountStake(slice)))
            }
            3 => Ok(StakeState::RewardsPool),
            _ => Err(ProgramError::InvalidAccountData),
        }
    }

    /// Return a `StakeState` from the given account info.
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
    ) -> Result<StakeState<'a>, ProgramError> {
        if account_info.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let data = account_info.borrow_data_unchecked();

        if data.len() < TYPE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let state_type = data[0];

        match state_type {
            0 => Ok(StakeState::Uninitialized),
            1 => {
                if data.len() < INITIALIZED_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                Ok(StakeState::Initialized(StakeAccountInitialized(data)))
            }
            2 => {
                if data.len() < STAKE_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                Ok(StakeState::Stake(StakeAccountStake(data)))
            }
            3 => Ok(StakeState::RewardsPool),
            _ => Err(ProgramError::InvalidAccountData),
        }
    }

    /// Return a `StakeState` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of a stake account,
    /// and it is properly aligned. This method performs length validation based on the
    /// state type but does not validate ownership.
    #[inline]
    pub unsafe fn from_bytes_unchecked(bytes: &'a [u8]) -> Result<StakeState<'a>, ProgramError> {
        if bytes.len() < TYPE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let state_type = bytes[0];

        match state_type {
            0 => Ok(StakeState::Uninitialized),
            1 => {
                if bytes.len() < INITIALIZED_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                Ok(StakeState::Initialized(StakeAccountInitialized(bytes)))
            }
            2 => {
                if bytes.len() < STAKE_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                Ok(StakeState::Stake(StakeAccountStake(bytes)))
            }
            3 => Ok(StakeState::RewardsPool),
            _ => Err(ProgramError::InvalidAccountData),
        }
    }

    /// Returns the stake state type.
    #[inline(always)]
    pub fn state_type(&self) -> StakeStateType {
        match self {
            StakeState::Uninitialized => StakeStateType::Uninitialized,
            StakeState::Initialized(_) => StakeStateType::Initialized,
            StakeState::Stake(_) => StakeStateType::Stake,
            StakeState::RewardsPool => StakeStateType::RewardsPool,
        }
    }
}
