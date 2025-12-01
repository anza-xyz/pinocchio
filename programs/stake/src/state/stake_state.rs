use core::ops::Deref;
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

/// Generic stake state enum that works with both borrowed and owned data.
///
/// This enum can hold either a `Ref<[u8]>` (safe, checked borrow) or `&[u8]`
/// (unchecked raw slice) depending on the type parameter.
#[repr(C)]
pub enum StakeState<T>
where
    T: Deref<Target = [u8]>,
{
    /// Account is not yet initialized.
    Uninitialized,
    /// Stake account is initialized but not delegated.
    Initialized(T),
    /// Stake account is delegated.
    Stake(T),
    /// Account is a rewards pool.
    RewardsPool,
}

impl<T> StakeState<T>
where
    T: Deref<Target = [u8]>,
{
    /// Returns a reference to the meta data if the account is Initialized or Stake.
    #[inline(always)]
    pub fn meta(&self) -> Option<&Meta> {
        match self {
            StakeState::Initialized(data) | StakeState::Stake(data) => {
                Some(unsafe { &*(data.as_ptr().add(META_OFFSET) as *const Meta) })
            }
            _ => None,
        }
    }

    /// Returns a reference to the stake data if the account is in Stake state.
    #[inline(always)]
    pub fn stake(&self) -> Option<&Stake> {
        match self {
            StakeState::Stake(data) => {
                Some(unsafe { &*(data.as_ptr().add(STAKE_OFFSET) as *const Stake) })
            }
            _ => None,
        }
    }
}

/// Safe implementation that borrows the account data and keeps the borrow alive.
impl<'a> TryFrom<&'a AccountInfo> for StakeState<Ref<'a, [u8]>> {
    type Error = ProgramError;

    fn try_from(account_info: &'a AccountInfo) -> Result<Self, Self::Error> {
        // Validate owner
        if !account_info.is_owned_by(&ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Borrow the account data
        let data = account_info.try_borrow_data()?;

        // Validate minimum length for type discriminator
        if data.len() < TYPE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        // Parse the state type and validate length
        match data[0] {
            0 => Ok(StakeState::Uninitialized),
            1 => {
                if data.len() < INITIALIZED_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                // Validate alignment for Meta struct
                if (data.as_ptr() as usize + META_OFFSET) % core::mem::align_of::<Meta>() != 0 {
                    return Err(ProgramError::InvalidAccountData);
                }
                Ok(StakeState::Initialized(data))
            }
            2 => {
                if data.len() < STAKE_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                // Validate alignment for Meta and Stake structs
                if (data.as_ptr() as usize + META_OFFSET) % core::mem::align_of::<Meta>() != 0
                    || (data.as_ptr() as usize + STAKE_OFFSET) % core::mem::align_of::<Stake>()
                        != 0
                {
                    return Err(ProgramError::InvalidAccountData);
                }
                Ok(StakeState::Stake(data))
            }
            3 => Ok(StakeState::RewardsPool),
            _ => Err(ProgramError::InvalidAccountData),
        }
    }
}

/// Unchecked implementation that works with raw byte slices.
///
/// # Safety
///
/// The caller must ensure that it is safe to access the data (e.g., there are
/// no mutable borrows of the account data).
impl<'a> TryFrom<&'a [u8]> for StakeState<&'a [u8]> {
    type Error = ProgramError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        // Validate minimum length for type discriminator
        if bytes.len() < TYPE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        // Parse the state type and validate length
        match bytes[0] {
            0 => Ok(StakeState::Uninitialized),
            1 => {
                if bytes.len() < INITIALIZED_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                // Validate alignment for Meta struct
                if (bytes.as_ptr() as usize + META_OFFSET) % core::mem::align_of::<Meta>() != 0 {
                    return Err(ProgramError::InvalidAccountData);
                }
                Ok(StakeState::Initialized(bytes))
            }
            2 => {
                if bytes.len() < STAKE_LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                // Validate alignment for Meta and Stake structs
                if (bytes.as_ptr() as usize + META_OFFSET) % core::mem::align_of::<Meta>() != 0
                    || (bytes.as_ptr() as usize + STAKE_OFFSET) % core::mem::align_of::<Stake>()
                        != 0
                {
                    return Err(ProgramError::InvalidAccountData);
                }
                Ok(StakeState::Stake(bytes))
            }
            3 => Ok(StakeState::RewardsPool),
            _ => Err(ProgramError::InvalidAccountData),
        }
    }
}
