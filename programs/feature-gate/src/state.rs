//! Feature account state.
//!
//! A [`Feature`] account tracks the activation status of a runtime feature
//! in the Solana cluster. It has the following 9-byte layout, matching
//! the bincode serialization of `Option<u64>`:
//!
//! | Offset | Size | Field                                        |
//! |:------:|:----:|:---------------------------------------------|
//! | `0`    | `1`  | `Option` tag — `0` for `None`, `1` for `Some`|
//! | `1`    | `8`  | Activation slot (little-endian `u64`)        |
//!
//! When the runtime activates a feature, the tag is set to `1` and the
//! activation slot is recorded at offset `1..9`.

use {
    solana_account_view::{AccountView, Ref},
    solana_program_error::ProgramError,
};

/// The tag byte value indicating `Option::Some` (feature activated).
const SOME_TAG: u8 = 1;

/// State of a feature account.
///
/// # Layout
///
/// This struct has a fixed serialized size of [`Feature::LEN`] bytes and
/// is `#[repr(C)]` with `align = 1`, making it safe to reinterpret over
/// the account data without copying.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Feature {
    /// `Option` tag: `0` (`None`) if the feature is pending activation,
    /// `1` (`Some`) if it has been activated by the runtime.
    tag: u8,
    /// Activation slot — only meaningful when [`Self::tag`] is `1`.
    ///
    /// Stored as little-endian bytes so the struct remains `align = 1`
    /// and zero-copy readable on unaligned boundaries.
    activation_slot: [u8; 8],
}

// Invariant: the serialized layout must be exactly 9 bytes to match the
// bincode encoding of `Option<u64>`.
const _: () = assert!(core::mem::size_of::<Feature>() == 9);
const _: () = assert!(core::mem::align_of::<Feature>() == 1);

impl Feature {
    /// The serialized length of a [`Feature`] account, in bytes.
    pub const LEN: usize = core::mem::size_of::<Self>();

    /// Returns a reference to the [`Feature`] at the start of `data`,
    /// without copying.
    ///
    /// # Errors
    ///
    /// Returns [`ProgramError::InvalidAccountData`] if `data` is shorter
    /// than [`Feature::LEN`] bytes.
    #[inline(always)]
    pub fn from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        // SAFETY: `Feature` is `#[repr(C)]` with `align = 1`, and `data`
        // is at least `Feature::LEN` bytes long.
        Ok(unsafe { &*(data.as_ptr() as *const Self) })
    }

    /// Returns a mutable reference to the [`Feature`] at the start of
    /// `data`, without copying.
    ///
    /// # Errors
    ///
    /// Returns [`ProgramError::InvalidAccountData`] if `data` is shorter
    /// than [`Feature::LEN`] bytes.
    #[inline(always)]
    pub fn from_bytes_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        // SAFETY: `Feature` is `#[repr(C)]` with `align = 1`, and `data`
        // is at least `Feature::LEN` bytes long.
        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    /// Borrows a [`Feature`] from the data of `account`.
    ///
    /// The returned [`Ref`] keeps the account data immutably borrowed for
    /// as long as it is alive, mirroring the borrow rules of
    /// [`AccountView::try_borrow`].
    ///
    /// # Errors
    ///
    /// Returns [`ProgramError::IncorrectProgramId`] if the account is
    /// not owned by the Feature Gate program, or
    /// [`ProgramError::InvalidAccountData`] if the account data is
    /// shorter than [`Feature::LEN`] bytes.
    #[inline(always)]
    pub fn from_account_view(account: &AccountView) -> Result<Ref<'_, Self>, ProgramError> {
        if !account.owned_by(&crate::ID) {
            return Err(ProgramError::IncorrectProgramId);
        }

        let data = account.try_borrow()?;
        if data.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Ref::map(data, |bytes| {
            // SAFETY: length checked above; `Feature` is `#[repr(C)]`
            // with `align = 1`.
            unsafe { &*(bytes.as_ptr() as *const Self) }
        }))
    }

    /// Returns the activation slot, or `None` if the feature has not
    /// been activated.
    #[inline(always)]
    pub fn activated_at(&self) -> Option<u64> {
        if self.tag == SOME_TAG {
            Some(u64::from_le_bytes(self.activation_slot))
        } else {
            None
        }
    }

    /// Returns `true` if the feature has been activated by the runtime.
    #[inline(always)]
    pub fn is_activated(&self) -> bool {
        self.tag == SOME_TAG
    }

    /// Returns the size a [`Feature`] account requires, in bytes.
    #[inline(always)]
    pub const fn size_of() -> usize {
        Self::LEN
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len_matches_bincode_option_u64() {
        // bincode encodes `Option<u64>` as 1 tag byte + 8 u64 bytes.
        assert_eq!(Feature::LEN, 9);
        assert_eq!(Feature::size_of(), 9);
    }

    #[test]
    fn from_bytes_rejects_short() {
        let too_short = [0u8; 8];
        assert!(matches!(
            Feature::from_bytes(&too_short),
            Err(ProgramError::InvalidAccountData)
        ));
    }

    #[test]
    fn activated_at_reads_little_endian_slot() {
        // Simulate a feature activated at slot 0x0102030405060708.
        let mut buf = [0u8; Feature::LEN];
        buf[0] = 1;
        buf[1..9].copy_from_slice(&0x0102030405060708u64.to_le_bytes());

        let feature = Feature::from_bytes(&buf).unwrap();
        assert!(feature.is_activated());
        assert_eq!(feature.activated_at(), Some(0x0102030405060708));
    }

    #[test]
    fn pending_feature_returns_none() {
        let buf = [0u8; Feature::LEN];
        let feature = Feature::from_bytes(&buf).unwrap();
        assert!(!feature.is_activated());
        assert_eq!(feature.activated_at(), None);
    }

    #[test]
    fn max_slot_round_trips() {
        let mut buf = [0u8; Feature::LEN];
        buf[0] = 1;
        buf[1..9].copy_from_slice(&u64::MAX.to_le_bytes());

        let feature = Feature::from_bytes(&buf).unwrap();
        assert_eq!(feature.activated_at(), Some(u64::MAX));
    }

    #[test]
    fn unknown_tag_treated_as_not_activated() {
        // bincode rejects any tag other than 0 or 1, but our zero-copy
        // reader is permissive — any non-1 tag is reported as not
        // activated rather than erroring, which keeps the hot path
        // branch-free.
        let mut buf = [0u8; Feature::LEN];
        buf[0] = 42;
        let feature = Feature::from_bytes(&buf).unwrap();
        assert_eq!(feature.activated_at(), None);
    }
}
