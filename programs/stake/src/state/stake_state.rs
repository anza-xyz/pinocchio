use pinocchio::{
    account_info::{AccountInfo, Ref},
    program_error::ProgramError,
};

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

impl TryFrom<u8> for StakeStateType {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(StakeStateType::Uninitialized),
            1 => Ok(StakeStateType::Initialized),
            2 => Ok(StakeStateType::Stake),
            3 => Ok(StakeStateType::RewardsPool),
            _ => Err(ProgramError::InvalidAccountData),
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

/// Zero-copy stake state wrapper that keeps the borrow alive.
///
/// This struct holds a `Ref` to the account data, ensuring the borrow
/// remains valid for the lifetime of the struct.
pub struct StakeState<'a> {
    data: Ref<'a, [u8]>,
    state_type: StakeStateType,
}

impl<'a> StakeState<'a> {
    /// Return a `StakeState` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safely borrowing
    /// the account data and keeping the borrow alive.
    #[inline]
    pub fn from_account_info(account_info: &'a AccountInfo) -> Result<Self, ProgramError> {
        if !account_info.is_owned_by(&ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let data = account_info.try_borrow_data()?;

        if data.len() < TYPE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let state_type = match data[0] {
            0 => StakeStateType::Uninitialized,
            1 => {
                if data.len() < INITIALIZED_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                StakeStateType::Initialized
            }
            2 => {
                if data.len() < STAKE_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                StakeStateType::Stake
            }
            3 => StakeStateType::RewardsPool,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Self { data, state_type })
    }

    /// Returns the stake state type.
    #[inline(always)]
    pub fn state_type(&self) -> StakeStateType {
        self.state_type
    }

    /// Returns a reference to the meta data if the account is Initialized or Stake.
    #[inline(always)]
    pub fn meta(&self) -> Option<&Meta> {
        match self.state_type {
            StakeStateType::Initialized | StakeStateType::Stake => {
                Some(unsafe { &*(self.data.as_ptr().add(META_OFFSET) as *const Meta) })
            }
            _ => None,
        }
    }

    /// Returns a reference to the stake data if the account is in Stake state.
    #[inline(always)]
    pub fn stake(&self) -> Option<&Stake> {
        match self.state_type {
            StakeStateType::Stake => {
                Some(unsafe { &*(self.data.as_ptr().add(STAKE_OFFSET) as *const Stake) })
            }
            _ => None,
        }
    }
}

/// Zero-copy stake state for unchecked access.
///
/// This struct provides direct access to stake account data without holding a borrow guard.
/// Use this when you need pattern matching or when working with raw bytes.
pub struct StakeStateUnchecked<'a> {
    data: &'a [u8],
    state_type: StakeStateType,
}

impl<'a> StakeStateUnchecked<'a> {
    /// Return a `StakeStateUnchecked` from the given account info.
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
    ) -> Result<Self, ProgramError> {
        if account_info.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Self::from_bytes(account_info.borrow_data_unchecked())
    }

    /// Return a `StakeStateUnchecked` from the given bytes.
    ///
    /// This method performs length validation based on the state type but does not
    /// validate ownership.
    #[inline]
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ProgramError> {
        if bytes.len() < TYPE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let state_type = match bytes[0] {
            0 => StakeStateType::Uninitialized,
            1 => {
                if bytes.len() < INITIALIZED_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                StakeStateType::Initialized
            }
            2 => {
                if bytes.len() < STAKE_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                StakeStateType::Stake
            }
            3 => StakeStateType::RewardsPool,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Self {
            data: bytes,
            state_type,
        })
    }

    /// Returns the stake state type.
    #[inline(always)]
    pub fn state_type(&self) -> StakeStateType {
        self.state_type
    }

    /// Returns a reference to the meta data if the account is Initialized or Stake.
    #[inline(always)]
    pub fn meta(&self) -> Option<&Meta> {
        match self.state_type {
            StakeStateType::Initialized | StakeStateType::Stake => {
                Some(unsafe { &*(self.data.as_ptr().add(META_OFFSET) as *const Meta) })
            }
            _ => None,
        }
    }

    /// Returns a reference to the stake data if the account is in Stake state.
    #[inline(always)]
    pub fn stake(&self) -> Option<&Stake> {
        match self.state_type {
            StakeStateType::Stake => {
                Some(unsafe { &*(self.data.as_ptr().add(STAKE_OFFSET) as *const Stake) })
            }
            _ => None,
        }
    }
}
