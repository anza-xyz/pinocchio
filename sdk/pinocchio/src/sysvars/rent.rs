//! This account contains the current cluster rent.
//!
//! This is required for the rent sysvar implementation.

use super::Sysvar;
use crate::{
    account_info::{AccountInfo, Ref},
    impl_sysvar_get,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// The ID of the rent sysvar.
pub const RENT_ID: Pubkey = [
    6, 167, 213, 23, 25, 44, 92, 81, 33, 140, 201, 76, 61, 74, 241, 127, 88, 218, 238, 8, 155, 161,
    253, 68, 227, 219, 217, 138, 0, 0, 0, 0,
];

/// Default lamports per byte cost.
///
/// This calculation is based on:
/// - `10^9` lamports per SOL
/// - `$1` per SOL
/// - `$0.01` per megabyte day
/// - `$3.65` per megabyte year
/// - `2` years period
pub const DEFAULT_LAMPORTS_PER_BYTE: u64 = 6960;

/// Account storage overhead for calculation of base rent.
///
/// This is the number of bytes required to store an account with no data. It is
/// added to an accounts data length when calculating [`Rent::minimum_balance`].
pub const ACCOUNT_STORAGE_OVERHEAD: u64 = 128;

/// Rent sysvar data.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Rent {
    /// Rental rate in lamports.
    _lamports_per_byte: u64,

    /// Exemption threshold in years.
    ///
    /// This was deprecated (see SIMD-0194). Use only `lamports_per_byte` instead.
    /// It was originally a `f64`, now represented as an array of 8 bytes.
    _exemption_threshold: [u8; 8],

    /// Burn percentage.
    _burn_percent: u8,
}

impl Rent {
    /// The length of the `Rent` sysvar account data.
    pub const LEN: usize = 8 + 8 + 1;

    /// Return a `Rent` from the given account info.
    ///
    /// This method performs a check on the account info key.
    ///
    /// # Important
    ///
    /// The values from the account are not used. The calculation is done following the
    /// changes proposed in [`SIMD-0194`].
    ///
    /// [`SIMD-0194`]: https://github.com/solana-foundation/solana-improvement-documents/pull/194
    #[inline]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<Ref<Rent>, ProgramError> {
        if account_info.key() != &RENT_ID {
            return Err(ProgramError::InvalidArgument);
        }
        Ok(Ref::map(account_info.try_borrow_data()?, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
    }

    /// Return a `Rent` from the given account info.
    ///
    /// This method performs a check on the account info key, but does not
    /// perform the borrow check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to borrow the account data - e.g., there are
    /// no mutable borrows of the account data.
    ///
    /// # Important
    ///
    /// The values from the account are not used. The calculation is done following the
    /// changes proposed in [`SIMD-0194`].
    ///
    /// [`SIMD-0194`]: https://github.com/solana-foundation/solana-improvement-documents/pull/194
    #[inline]
    pub unsafe fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&Self, ProgramError> {
        if account_info.key() != &RENT_ID {
            return Err(ProgramError::InvalidArgument);
        }
        Ok(Self::from_bytes_unchecked(
            account_info.borrow_data_unchecked(),
        ))
    }

    /// Return a `Rent` from the given bytes.
    ///
    /// This method performs a length validation. The caller must ensure that `bytes` contains
    /// a valid representation of `Rent`.
    ///
    /// # Important
    ///
    /// The values from the rent bytes are not used. The calculation is done following the
    /// changes proposed in [`SIMD-0194`].
    ///
    /// [`SIMD-0194`]: https://github.com/solana-foundation/solana-improvement-documents/pull/194
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() < Self::LEN {
            return Err(ProgramError::InvalidArgument);
        }
        // SAFETY: `bytes` has been validated to be at least `Self::LEN` bytes long; the
        // caller must ensure that `bytes` contains a valid representation of `Rent`.
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Return a `Rent` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `Rent` and
    /// that is has the expected length.
    ///
    /// # Important
    ///
    /// The values from the rent bytes are not used. The calculation is done following the
    /// changes proposed in [`SIMD-0194`].
    ///
    /// [`SIMD-0194`]: https://github.com/solana-foundation/solana-improvement-documents/pull/194
    #[inline]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const Rent)
    }

    /// Calculates the minimum balance for rent exemption.
    ///
    /// This method avoids floating-point operations when the `exemption_threshold`
    /// is the default value.
    ///
    /// # Arguments
    ///
    /// * `data_len` - The number of bytes in the account
    ///
    /// # Returns
    ///
    /// The minimum balance in lamports for rent exemption.
    #[inline]
    pub fn minimum_balance(&self, data_len: usize) -> u64 {
        (ACCOUNT_STORAGE_OVERHEAD + data_len as u64) * DEFAULT_LAMPORTS_PER_BYTE
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
    #[inline]
    pub fn is_exempt(&self, lamports: u64, data_len: usize) -> bool {
        lamports >= self.minimum_balance(data_len)
    }
}

impl Sysvar for Rent {
    impl_sysvar_get!(sol_get_rent_sysvar);
}

/// The return value of [`Rent::due`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RentDue {
    /// Used to indicate the account is rent exempt.
    Exempt,
    /// The account owes this much rent.
    Paying(u64),
}

impl RentDue {
    /// Return the lamports due for rent.
    pub fn lamports(&self) -> u64 {
        match self {
            RentDue::Exempt => 0,
            RentDue::Paying(x) => *x,
        }
    }

    /// Return 'true' if rent exempt.
    pub fn is_exempt(&self) -> bool {
        match self {
            RentDue::Exempt => true,
            RentDue::Paying(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sysvars::rent::ACCOUNT_STORAGE_OVERHEAD;

    /// Deprecated: previous values used in the rent calculation.
    const DEFAULT_LAMPORTS_PER_BYTE_YEAR: u64 = 1_000_000_000 / 100 * 365 / (1024 * 1024);

    /// Deprecated: previous values used in the rent calculation.
    const DEFAULT_EXEMPTION_THRESHOLD: f64 = 2.0;

    #[test]
    pub fn test_minimum_balance() {
        // The struct values should not be not used.
        let mut rent = super::Rent {
            _lamports_per_byte: 0,
            _exemption_threshold: [0; 8],
            _burn_percent: 0,
        };

        // Using the default exemption threshold.

        let balance = rent.minimum_balance(100);
        let calculated = (((ACCOUNT_STORAGE_OVERHEAD + 100) * DEFAULT_LAMPORTS_PER_BYTE_YEAR)
            as f64
            * DEFAULT_EXEMPTION_THRESHOLD) as u64;

        assert!(calculated > 0);
        assert_eq!(balance, calculated);

        // Using a different exemption threshold.
        rent._lamports_per_byte = 1000;

        let balance = rent.minimum_balance(500);
        let calculated = (((ACCOUNT_STORAGE_OVERHEAD + 500) * DEFAULT_LAMPORTS_PER_BYTE_YEAR)
            as f64
            * DEFAULT_EXEMPTION_THRESHOLD) as u64;

        assert!(calculated > 0);
        assert_eq!(balance, calculated);
    }
}
