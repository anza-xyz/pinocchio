use core::mem::size_of;
use pinocchio::{
    account_info::{AccountInfo, Ref},
    Address,
    error::ProgramError,
};

use crate::{instructions::MAX_MULTISIG_SIGNERS, ID};

/// Multisignature data.
#[repr(C)]
pub struct Multisig {
    /// Number of signers required
    m: u8,
    /// Number of valid signers
    n: u8,
    /// Is `true` if this structure has been initialized
    is_initialized: u8,
    /// Signer public keys
    signers: [Address; MAX_MULTISIG_SIGNERS],
}

impl Multisig {
    /// The length of the `Multisig` account data.
    pub const LEN: usize = size_of::<Multisig>();

    /// Return a `Multisig` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<Ref<Multisig>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if !account_info.is_owned_by(&ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(Ref::map(account_info.try_borrow_data()?, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
    }

    /// Return a `Multisig` from the given account info.
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
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(Self::from_bytes_unchecked(
            account_info.borrow_data_unchecked(),
        ))
    }

    /// Return a `Multisig` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `Multisig`, and
    /// it has the correct length to be interpreted as an instance of `Multisig`.
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const Multisig)
    }

    /// Number of signers required to validate the `Multisig` signature.
    #[inline(always)]
    pub const fn required_signers(&self) -> u8 {
        self.m
    }

    /// Number of signer addresses present on the `Multisig`.
    #[inline(always)]
    pub const fn signers_len(&self) -> usize {
        self.n as usize
    }

    /// Return the signer addresses of the `Multisig`.
    #[inline(always)]
    pub fn signers(&self) -> &[Address] {
        // SAFETY: `self.signers` is an array of `Address` with a fixed size of
        // `MAX_MULTISIG_SIGNERS`; `self.signers_len` is always `<= MAX_MULTISIG_SIGNERS`
        // and indicates how many of these signers are valid.
        unsafe { self.signers.get_unchecked(..self.signers_len()) }
    }

    /// Check whether the multisig is initialized or not.
    //
    // It will return a boolean value indicating whether [`self.is_initialized`]
    // is different than `0` or not.
    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.is_initialized != 0
    }
}
