//! This account contains the current cluster rent.
//!
//! This is required for the rent sysvar implementation.
use {
    crate::{
        account::AccountView, error::ProgramError, hint::unlikely, impl_sysvar_get,
        sysvars::Sysvar, Address,
    },
    core::mem::{align_of, size_of},
};

/// The ID of the rent sysvar.
pub const RENT_ID: Address = Address::new_from_array([
    6, 167, 213, 23, 25, 44, 92, 81, 33, 140, 201, 76, 61, 74, 241, 127, 88, 218, 238, 8, 155, 161,
    253, 68, 227, 219, 217, 138, 0, 0, 0, 0,
]);

/// Maximum permitted size of account data (10 MiB).
const MAX_PERMITTED_DATA_LENGTH: u64 = 10 * 1024 * 1024;

/// Default rental rate in lamports/byte.
///
/// This calculation is based on:
/// - `10^9` lamports per SOL
/// - `$1` per SOL
/// - `$0.01` per megabyte day
/// - `$7.30` per megabyte
pub const DEFAULT_LAMPORTS_PER_BYTE: u64 = 6960;

/// Account storage overhead for calculation of base rent.
///
/// This is the number of bytes required to store an account with no data. It is
/// added to an accounts data length when calculating [`Rent::minimum_balance`].
pub const ACCOUNT_STORAGE_OVERHEAD: u64 = 128;

/// Maximum lamports per byte value.
const MAX_LAMPORTS_PER_BYTE: u64 = 1_759_197_129_867;

/// Rent sysvar data
#[repr(C)]
#[cfg_attr(feature = "copy", derive(Copy))]
#[derive(Clone, Debug)]
pub struct Rent {
    /// Rental rate in lamports per byte.
    lamports_per_byte: u64,
}

// Assert that the size of the `Rent` struct is as expected (8 bytes).
const _ASSERT_STRUCT_LEN: () = assert!(size_of::<Rent>() == 8);

// Assert that the alignment of the `Rent` struct is as expected (8 byte).
const _ASSERT_ACCOUNT_ALIGN: () = assert!(align_of::<Rent>() == 8);

impl Rent {
    /// Return a `Rent` from the given account view.
    ///
    /// This method performs a check on the account view key.
    #[inline]
    pub fn from_account_view(account_view: &AccountView) -> Result<Rent, ProgramError> {
        if unlikely(account_view.is_borrowed_mut()) {
            return Err(ProgramError::AccountBorrowFailed);
        }

        // SAFETY: The account data can be safely borrowed.
        let rent = unsafe { Self::from_account_view_unchecked(account_view) }?;
        Ok(rent)
    }

    /// Return a `Rent` from the given account view.
    ///
    /// This method performs a check on the account view key, but does not
    /// perform the borrow check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to borrow the account data -
    /// e.g., there are no mutable borrows of the account data.
    #[inline]
    pub unsafe fn from_account_view_unchecked(
        account_view: &AccountView,
    ) -> Result<Self, ProgramError> {
        if unlikely(account_view.address() != &RENT_ID) {
            return Err(ProgramError::InvalidArgument);
        }
        Ok(Self::from_bytes_unchecked(account_view.borrow_unchecked()))
    }

    /// Return a `Rent` from the given bytes.
    ///
    /// This method performs a length validation. The caller must
    /// ensure that `bytes` contains a valid representation of `Rent`.
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProgramError> {
        if bytes.len() < size_of::<Self>() {
            return Err(ProgramError::InvalidArgument);
        }

        // SAFETY: `bytes` has the expected length.
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Return a `Rent` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of
    /// `Rent` and that is has the expected length.
    #[inline]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> Self {
        Self {
            // SAFETY: The caller must ensure that `bytes` has the expected length.
            lamports_per_byte: unsafe {
                core::ptr::read_unaligned::<u64>(bytes.as_ptr() as *const u64)
            },
        }
    }

    /// Calculates the minimum balance for rent exemption.
    ///
    /// This method avoids floating-point operations when the
    /// `exemption_threshold` is the default value.
    ///
    /// # Arguments
    ///
    /// * `data_len` - The number of bytes in the account
    ///
    /// # Returns
    ///
    /// The minimum balance in lamports for rent exemption.
    ///
    /// # Panics
    ///
    /// Panics if `data_len` exceeds the maximum permitted data length or if the
    /// `lamports_per_byte` is too large based on the `exemption_threshold`.
    #[deprecated(since = "0.10.0", note = "Use `Rent::try_minimum_balance` instead")]
    #[inline(always)]
    pub fn minimum_balance(&self, data_len: usize) -> u64 {
        self.try_minimum_balance(data_len)
            .expect("Maximum permitted data length exceeded")
    }

    /// Calculates the minimum balance for rent exemption without performing
    /// any validation.
    ///
    /// # Important
    ///
    /// The caller must ensure that `data_len` is within the permitted limit
    /// and the `lamports_per_byte` is within the permitted limit based on
    /// the `exemption_threshold` to avoid overflow.
    ///
    /// # Arguments
    ///
    /// * `data_len` - The number of bytes in the account
    ///
    /// # Returns
    ///
    /// The minimum balance in lamports for rent exemption.
    #[inline(always)]
    pub fn minimum_balance_unchecked(&self, data_len: usize) -> u64 {
        (ACCOUNT_STORAGE_OVERHEAD + data_len as u64) * self.lamports_per_byte
    }

    /// Calculates the minimum balance for rent exemption.
    ///
    /// This method avoids floating-point operations when the
    /// `exemption_threshold` is the default value.
    ///
    /// # Arguments
    ///
    /// * `data_len` - The number of bytes in the account
    ///
    /// # Returns
    ///
    /// The minimum balance in lamports for rent exemption.
    ///
    /// # Errors
    ///
    /// Returns `ProgramError::InvalidArgument` if `data_len` exceeds the
    /// maximum permitted data length or if the `lamports_per_byte` is too
    /// large based on the `exemption_threshold`, which would cause an
    /// overflow.
    #[inline(always)]
    pub fn try_minimum_balance(&self, data_len: usize) -> Result<u64, ProgramError> {
        if data_len as u64 > MAX_PERMITTED_DATA_LENGTH {
            return Err(ProgramError::InvalidArgument);
        }

        // Validate `lamports_per_byte` based on `exemption_threshold` to prevent
        // overflow.

        if unlikely(self.lamports_per_byte > MAX_LAMPORTS_PER_BYTE) {
            return Err(ProgramError::InvalidArgument);
        }

        Ok(self.minimum_balance_unchecked(data_len))
    }

    /// Determines if an account can be considered rent exempt.
    ///
    /// # Arguments
    ///
    /// * `lamports` - The balance of the account in lamports
    /// * `data_len` - The size of the account in bytes
    ///
    /// # Returns
    ///
    /// `true`` if the account is rent exempt, `false`` otherwise.
    #[allow(deprecated)]
    #[inline]
    pub fn is_exempt(&self, lamports: u64, data_len: usize) -> bool {
        lamports >= self.minimum_balance(data_len)
    }
}

impl Sysvar for Rent {
    impl_sysvar_get!(RENT_ID, 0);
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use crate::sysvars::rent::{ACCOUNT_STORAGE_OVERHEAD, DEFAULT_LAMPORTS_PER_BYTE};

    #[test]
    pub fn test_minimum_balance() {
        // Happy path
        let mut rent = super::Rent {
            lamports_per_byte: DEFAULT_LAMPORTS_PER_BYTE,
        };

        let balance = rent.minimum_balance(100);
        let calculated = (ACCOUNT_STORAGE_OVERHEAD + 100) * rent.lamports_per_byte;

        assert!(calculated > 0);
        assert_eq!(balance, calculated);

        // Using diferent lamports per byte value
        rent.lamports_per_byte = DEFAULT_LAMPORTS_PER_BYTE * 2;

        let balance = rent.minimum_balance(100);
        let calculated = (ACCOUNT_STORAGE_OVERHEAD + 100) * rent.lamports_per_byte;

        assert!(calculated > 0);
        assert_eq!(balance, calculated);
    }

    #[test]
    pub fn test_from_bytes() {
        // Happy Path
        let rent = super::Rent {
            lamports_per_byte: DEFAULT_LAMPORTS_PER_BYTE,
        };

        let bytes = rent.lamports_per_byte.to_le_bytes();
        let deserialized = super::Rent::from_bytes(&bytes).unwrap();

        assert_eq!(rent.lamports_per_byte, deserialized.lamports_per_byte);

        // Invalid length

        let bytes = [0u8; 7];
        let result = super::Rent::from_bytes(&bytes);
        assert!(result.is_err());
    }
}
