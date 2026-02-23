//! This account contains the current cluster rent.
//!
//! This is required for the rent sysvar implementation.

use {
    crate::{
        account::{AccountView, Ref},
        error::ProgramError,
        hint::unlikely,
        impl_sysvar_get,
        sysvars::Sysvar,
        Address,
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

/// Default rental rate in lamports/byte-year.
///
/// This calculation is based on:
/// - `10^9` lamports per SOL
/// - `$1` per SOL
/// - `$0.01` per megabyte day
/// - `$3.65` per megabyte year
#[deprecated(
    since = "0.10.0",
    note = "The concept of rent no longer exists, only rent-exemption. Use \
            `DEFAULT_LAMPORTS_PER_BYTE` instead"
)]
pub const DEFAULT_LAMPORTS_PER_BYTE_YEAR: u64 = 1_000_000_000 / 100 * 365 / (1024 * 1024);

/// Default rental rate in lamports/byte.
///
/// This calculation is based on:
/// - `10^9` lamports per SOL
/// - `$1` per SOL
/// - `$0.01` per megabyte day
/// - `$7.30` per megabyte
pub const DEFAULT_LAMPORTS_PER_BYTE: u64 = 6960;

#[cfg(not(target_arch = "bpf"))]
/// Default amount of time (in years) the balance has to include rent for the
/// account to be rent exempt.
#[deprecated(
    since = "0.10.0",
    note = "The concept of rent no longer exists, only rent-exemption"
)]
pub const DEFAULT_EXEMPTION_THRESHOLD: f64 = 2.0;

/// The `f64::to_le_bytes` representation of the default exemption threshold.
///
/// This value is equivalent to `2.0f64`. It is only used to check whether
/// the exemption threshold is the default value to avoid performing
/// floating-point operations on-chain.
const CURRENT_EXEMPTION_THRESHOLD: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 64];

/// The `f64::to_le_bytes` representation of the SIMD-0194 exemption threshold.
///
/// This value is equivalent to `1.0f64`. It is only used to check whether
/// the exemption threshold is the deprecated value to avoid performing
/// floating-point operations on-chain.
const SIMD0194_EXEMPTION_THRESHOLD: [u8; 8] = [0, 0, 0, 0, 0, 0, 240, 63];

/// Default percentage of collected rent that is burned.
///
/// Valid values are in the range [0, 100]. The remaining percentage is
/// distributed to validators.
#[deprecated(
    since = "0.10.0",
    note = "The concept of rent no longer exists, only rent-exemption"
)]
pub const DEFAULT_BURN_PERCENT: u8 = 50;

/// Account storage overhead for calculation of base rent.
///
/// This is the number of bytes required to store an account with no data. It is
/// added to an accounts data length when calculating [`Rent::minimum_balance`].
pub const ACCOUNT_STORAGE_OVERHEAD: u64 = 128;

/// Maximum lamports per byte for the SIMD-0194 exemption threshold.
const SIMD0194_MAX_LAMPORTS_PER_BYTE: u64 = 1_759_197_129_867;

/// Maximum lamports per byte for the current exemption threshold.
const CURRENT_MAX_LAMPORTS_PER_BYTE: u64 = 879_598_564_933;

/// Rent sysvar data
#[repr(C)]
#[cfg_attr(feature = "copy", derive(Copy))]
#[derive(Clone, Debug)]
pub struct Rent {
    /// Rental rate in lamports per byte.
    lamports_per_byte: u64,

    /// Exemption threshold in years.
    ///
    /// The concept of rent no longer exists, only rent-exemption.
    exemption_threshold: [u8; 8],
    // Burn percentage.
    //
    // The concept of rent no longer exists, only rent-exemption.
    //
    // The field (`burn_percent: u8`) is omitted to maintain the struct
    // aligned to 8-bytes, which improves the performance of loading the
    // sysvar.
}

// Assert that the size of the `Rent` struct is as expected (16 bytes).
const _ASSERT_STRUCT_LEN: () = assert!(size_of::<Rent>() == 16);

// Assert that the alignment of the `Rent` struct is as expected (8 byte).
const _ASSERT_ACCOUNT_ALIGN: () = assert!(align_of::<Rent>() == 8);

impl Rent {
    /// Return a `Rent` from the given account view.
    ///
    /// This method performs a check on the account view key.
    #[inline]
    pub fn from_account_view(account_view: &AccountView) -> Result<Ref<Rent>, ProgramError> {
        if unlikely(account_view.address() != &RENT_ID) {
            return Err(ProgramError::InvalidArgument);
        }
        // SAFETY: Account data for Rent sysvar has fixed layout; we validated the address and borrow.
        Ok(Ref::map(account_view.try_borrow()?, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
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
    ) -> Result<&Self, ProgramError> {
        if unlikely(account_view.address() != &RENT_ID) {
            return Err(ProgramError::InvalidArgument);
        }
        Ok(Self::from_bytes_unchecked(account_view.borrow_unchecked()))
    }

    /// Return a `Rent` from the given bytes.
    ///
    /// This method performs a length and alignment validation. The caller must
    /// ensure that `bytes` contains a valid representation of `Rent`.
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() < size_of::<Self>() {
            return Err(ProgramError::InvalidArgument);
        }
        if bytes.as_ptr().align_offset(align_of::<Rent>()) != 0 {
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
    /// The caller must ensure that `bytes` contains a valid representation of
    /// `Rent` and that is has the expected length.
    #[inline]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const Rent)
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
    /// This method avoids floating-point operations when the
    /// `exemption_threshold` is the default value.
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
        let bytes = data_len as u64;

        // There are two cases where it is possible to avoid floating-point
        // operations:
        //
        //   1) exemption threshold is `1.0` (the SIMD-0194 default)
        //   2) exemption threshold is `2.0` (the current default)
        //
        // In all other cases, perform the full calculation using floating-point
        // operations. Note that on BPF targets, floating-point operations are
        // not supported, so panic in that case.
        if self.exemption_threshold == SIMD0194_EXEMPTION_THRESHOLD {
            (ACCOUNT_STORAGE_OVERHEAD + bytes) * self.lamports_per_byte
        } else if self.exemption_threshold == CURRENT_EXEMPTION_THRESHOLD {
            2 * (ACCOUNT_STORAGE_OVERHEAD + bytes) * self.lamports_per_byte
        } else {
            #[cfg(not(target_arch = "bpf"))]
            {
                (((ACCOUNT_STORAGE_OVERHEAD + bytes) * self.lamports_per_byte) as f64
                    * f64::from_le_bytes(self.exemption_threshold)) as u64
            }
            #[cfg(target_arch = "bpf")]
            panic!("Floating-point operations are not supported on BPF targets");
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
    /// # Errors
    ///
    /// Returns `ProgramError::InvalidArgument` if `data_len` exceeds the
    /// maximum permitted data length or if the `lamports_per_byte` is too
    /// large based on the `exemption_threshold`, which would cause an
    /// overflow.
    //
    // Note: Clippy suggests collapsing the `if` statements, but they are kept
    // separate since it is more CU-efficient this way.
    #[allow(clippy::collapsible_if)]
    #[inline(always)]
    pub fn try_minimum_balance(&self, data_len: usize) -> Result<u64, ProgramError> {
        if data_len as u64 > MAX_PERMITTED_DATA_LENGTH {
            return Err(ProgramError::InvalidArgument);
        }

        // Validate `lamports_per_byte` based on `exemption_threshold`
        // to prevent overflow.

        if unlikely(self.lamports_per_byte > CURRENT_MAX_LAMPORTS_PER_BYTE) {
            if self.exemption_threshold == CURRENT_EXEMPTION_THRESHOLD {
                return Err(ProgramError::InvalidArgument);
            }
        } else if unlikely(self.lamports_per_byte > SIMD0194_MAX_LAMPORTS_PER_BYTE) {
            if self.exemption_threshold == SIMD0194_EXEMPTION_THRESHOLD {
                return Err(ProgramError::InvalidArgument);
            }
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
    #[inline]
    pub fn is_exempt(&self, lamports: u64, data_len: usize) -> bool {
        self.try_minimum_balance(data_len)
            .map(|min| lamports >= min)
            .unwrap_or(false)
    }
}

impl Sysvar for Rent {
    impl_sysvar_get!(RENT_ID, 0);
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use crate::sysvars::rent::{
        ACCOUNT_STORAGE_OVERHEAD, CURRENT_EXEMPTION_THRESHOLD, DEFAULT_LAMPORTS_PER_BYTE,
        DEFAULT_LAMPORTS_PER_BYTE_YEAR, SIMD0194_EXEMPTION_THRESHOLD,
    };

    #[test]
    pub fn test_minimum_balance() {
        let mut rent = super::Rent {
            lamports_per_byte: DEFAULT_LAMPORTS_PER_BYTE_YEAR,
            exemption_threshold: CURRENT_EXEMPTION_THRESHOLD,
            //burn_percent: DEFAULT_BURN_PERCENT,
        };

        // Using the default exemption threshold.

        let balance = rent.minimum_balance(100);
        let calculated = (((ACCOUNT_STORAGE_OVERHEAD + 100) * rent.lamports_per_byte) as f64
            * f64::from_le_bytes(rent.exemption_threshold)) as u64;

        assert!(calculated > 0);
        assert_eq!(balance, calculated);

        // Using a different exemption threshold.

        rent.exemption_threshold = 0.5f64.to_le_bytes();

        let balance = rent.minimum_balance(100);
        let calculated = (((ACCOUNT_STORAGE_OVERHEAD + 100) * rent.lamports_per_byte) as f64
            * f64::from_le_bytes(rent.exemption_threshold)) as u64;

        assert!(calculated > 0);
        assert_eq!(balance, calculated);
    }

    #[test]
    pub fn test_minimum_balance_simd0194() {
        let mut rent = super::Rent {
            lamports_per_byte: DEFAULT_LAMPORTS_PER_BYTE,
            exemption_threshold: SIMD0194_EXEMPTION_THRESHOLD,
            //burn_percent: DEFAULT_BURN_PERCENT,
        };

        // Using the default exemption threshold.

        let balance = rent.minimum_balance(100);
        let calculated = (ACCOUNT_STORAGE_OVERHEAD + 100) * rent.lamports_per_byte;

        assert!(calculated > 0);
        assert_eq!(balance, calculated);

        // Using a different lamports per byte value.

        rent.lamports_per_byte = DEFAULT_LAMPORTS_PER_BYTE * 2;

        let balance = rent.minimum_balance(100);
        let calculated = (ACCOUNT_STORAGE_OVERHEAD + 100) * rent.lamports_per_byte;

        assert!(calculated > 0);
        assert_eq!(balance, calculated);
    }
}
