//! Configuration for epochs and leader schedules.
//!
//! This sysvar describes the length of epochs and the warmup schedule the
//! cluster uses when computing leader schedules.
//!
//! # Serialized layout (33 bytes, bincode)
//!
//! | Offset | Size | Field                          |
//! |:------:|:----:|:-------------------------------|
//! | `0`    | `8`  | `slots_per_epoch`              |
//! | `8`    | `8`  | `leader_schedule_slot_offset`  |
//! | `16`   | `1`  | `warmup`                       |
//! | `17`   | `8`  | `first_normal_epoch`           |
//! | `25`   | `8`  | `first_normal_slot`            |
//!
//! The bincode layout does not match a natural `#[repr(C)]` layout (the
//! `bool` is sandwiched between `u64`s), so the data is stored as a raw
//! byte array and accessors perform unaligned reads. This is the same
//! shape the runtime uses when writing the sysvar via `sol_get_sysvar`.

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

/// The ID of the epoch schedule sysvar.
pub const EPOCH_SCHEDULE_ID: Address = Address::new_from_array([
    6, 167, 213, 23, 24, 220, 63, 238, 2, 211, 228, 127, 1, 0, 248, 176, 84, 247, 148, 46, 96, 89,
    30, 63, 80, 135, 25, 168, 5, 0, 0, 0,
]);

/// The default number of slots in each epoch.
pub const DEFAULT_SLOTS_PER_EPOCH: u64 = 432_000;

/// The default leader schedule slot offset.
pub const DEFAULT_LEADER_SCHEDULE_SLOT_OFFSET: u64 = DEFAULT_SLOTS_PER_EPOCH;

/// The maximum number of epochs beyond the current one that can have a
/// leader schedule calculated.
pub const MAX_LEADER_SCHEDULE_EPOCH_OFFSET: u64 = 3;

/// The minimum number of slots in an epoch.
pub const MINIMUM_SLOTS_PER_EPOCH: u64 = 32;

/// Configuration for epochs and leader schedules.
///
/// # Layout
///
/// The struct wraps a 33-byte buffer that exactly matches the bincode
/// serialization the runtime writes into the sysvar account. Field
/// accessors return owned values via unaligned reads, so callers never
/// observe the underlying byte layout.
#[repr(C)]
#[cfg_attr(feature = "copy", derive(Copy))]
#[derive(Clone, Debug)]
pub struct EpochSchedule {
    data: [u8; Self::LEN],
}

// Assert that the size of the `EpochSchedule` struct is as expected (33
// bytes, matching the bincode encoding).
const _ASSERT_STRUCT_LEN: () = assert!(size_of::<EpochSchedule>() == 33);

// Assert that the alignment of the `EpochSchedule` struct is as expected
// (1 byte), so unaligned reads remain safe regardless of the underlying
// account data alignment.
const _ASSERT_ACCOUNT_ALIGN: () = assert!(align_of::<EpochSchedule>() == 1);

impl EpochSchedule {
    /// The length of the `EpochSchedule` sysvar account data, in bytes.
    pub const LEN: usize = 33;

    // Field offsets within `data`.
    const OFFSET_SLOTS_PER_EPOCH: usize = 0;
    const OFFSET_LEADER_SCHEDULE_SLOT_OFFSET: usize = 8;
    const OFFSET_WARMUP: usize = 16;
    const OFFSET_FIRST_NORMAL_EPOCH: usize = 17;
    const OFFSET_FIRST_NORMAL_SLOT: usize = 25;

    /// Return an `EpochSchedule` from the given account view.
    ///
    /// This method performs a check on the account view address.
    #[inline]
    pub fn from_account_view(
        account_view: &AccountView,
    ) -> Result<Ref<'_, EpochSchedule>, ProgramError> {
        if unlikely(account_view.address() != &EPOCH_SCHEDULE_ID) {
            return Err(ProgramError::InvalidArgument);
        }
        Ok(Ref::map(account_view.try_borrow()?, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
    }

    /// Return an `EpochSchedule` from the given account view.
    ///
    /// This method performs a check on the account view address, but does
    /// not perform the borrow check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to borrow the account data
    /// - e.g., there are no mutable borrows of the account data.
    #[inline]
    pub unsafe fn from_account_view_unchecked(
        account_view: &AccountView,
    ) -> Result<&Self, ProgramError> {
        if unlikely(account_view.address() != &EPOCH_SCHEDULE_ID) {
            return Err(ProgramError::InvalidArgument);
        }
        Ok(Self::from_bytes_unchecked(account_view.borrow_unchecked()))
    }

    /// Return an `EpochSchedule` from the given bytes.
    ///
    /// Performs a length validation. The caller must ensure that `bytes`
    /// contains a valid representation of `EpochSchedule`.
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() < Self::LEN {
            return Err(ProgramError::InvalidArgument);
        }
        // SAFETY: `bytes` has been validated to be at least `Self::LEN`
        // bytes long, and `EpochSchedule` has `align = 1`.
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Return an `EpochSchedule` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation
    /// of `EpochSchedule` and that it has the expected length.
    #[inline]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const EpochSchedule)
    }

    /// The maximum number of slots in each epoch.
    #[inline(always)]
    pub fn slots_per_epoch(&self) -> u64 {
        Self::read_u64(&self.data, Self::OFFSET_SLOTS_PER_EPOCH)
    }

    /// A number of slots before beginning of an epoch to calculate
    /// a leader schedule for that epoch.
    #[inline(always)]
    pub fn leader_schedule_slot_offset(&self) -> u64 {
        Self::read_u64(&self.data, Self::OFFSET_LEADER_SCHEDULE_SLOT_OFFSET)
    }

    /// Whether epochs start short and grow.
    #[inline(always)]
    pub fn warmup(&self) -> bool {
        self.data[Self::OFFSET_WARMUP] != 0
    }

    /// The first epoch after the warmup period.
    ///
    /// Basically: `MINIMUM_SLOTS_PER_EPOCH.trailing_zeros()`
    ///        - `slots_per_epoch.next_power_of_two().trailing_zeros()`.
    #[inline(always)]
    pub fn first_normal_epoch(&self) -> u64 {
        Self::read_u64(&self.data, Self::OFFSET_FIRST_NORMAL_EPOCH)
    }

    /// The first slot after the warmup period.
    ///
    /// Basically: `MINIMUM_SLOTS_PER_EPOCH * (2.pow(first_normal_epoch) - 1)`.
    #[inline(always)]
    pub fn first_normal_slot(&self) -> u64 {
        Self::read_u64(&self.data, Self::OFFSET_FIRST_NORMAL_SLOT)
    }

    /// Read a little-endian `u64` at `offset` from `bytes`.
    ///
    /// The read is unaligned, so it is safe regardless of `bytes`'s
    /// alignment.
    #[inline(always)]
    fn read_u64(bytes: &[u8; Self::LEN], offset: usize) -> u64 {
        // SAFETY: caller-enforced invariant — offsets are static constants
        // and all `u64` fields are fully contained within `Self::LEN`.
        unsafe { core::ptr::read_unaligned::<u64>(bytes.as_ptr().add(offset) as *const u64) }
    }

    /// Returns the number of slots in the given epoch.
    pub fn get_slots_in_epoch(&self, epoch: u64) -> u64 {
        if epoch < self.first_normal_epoch() {
            2u64.saturating_pow(
                (epoch as u32).saturating_add(MINIMUM_SLOTS_PER_EPOCH.trailing_zeros()),
            )
        } else {
            self.slots_per_epoch()
        }
    }

    /// Returns the epoch for which the given slot should have its leader
    /// schedule calculated.
    pub fn get_leader_schedule_epoch(&self, slot: u64) -> u64 {
        let first_normal_slot = self.first_normal_slot();
        if slot < first_normal_slot {
            self.get_epoch_and_slot_index(slot).0.saturating_add(1)
        } else {
            let new_slots_since_first_normal_slot = slot.saturating_sub(first_normal_slot);
            let new_first_normal_leader_schedule_slot = new_slots_since_first_normal_slot
                .saturating_add(self.leader_schedule_slot_offset());
            let new_epochs_since_first_normal_leader_schedule =
                new_first_normal_leader_schedule_slot
                    .checked_div(self.slots_per_epoch())
                    .unwrap_or(0);
            self.first_normal_epoch()
                .saturating_add(new_epochs_since_first_normal_leader_schedule)
        }
    }

    /// Returns the epoch that contains the given slot.
    #[inline(always)]
    pub fn get_epoch(&self, slot: u64) -> u64 {
        self.get_epoch_and_slot_index(slot).0
    }

    /// Returns the epoch and slot index (position within the epoch) for
    /// the given slot.
    pub fn get_epoch_and_slot_index(&self, slot: u64) -> (u64, u64) {
        let first_normal_slot = self.first_normal_slot();
        if slot < first_normal_slot {
            let epoch = slot
                .saturating_add(MINIMUM_SLOTS_PER_EPOCH)
                .saturating_add(1)
                .next_power_of_two()
                .trailing_zeros()
                .saturating_sub(MINIMUM_SLOTS_PER_EPOCH.trailing_zeros())
                .saturating_sub(1);

            let epoch_len =
                2u64.saturating_pow(epoch.saturating_add(MINIMUM_SLOTS_PER_EPOCH.trailing_zeros()));

            (
                u64::from(epoch),
                slot.saturating_sub(epoch_len.saturating_sub(MINIMUM_SLOTS_PER_EPOCH)),
            )
        } else {
            let slots_per_epoch = self.slots_per_epoch();
            let normal_slot_index = slot.saturating_sub(first_normal_slot);
            let normal_epoch_index = normal_slot_index.checked_div(slots_per_epoch).unwrap_or(0);
            let epoch = self.first_normal_epoch().saturating_add(normal_epoch_index);
            let slot_index = normal_slot_index.checked_rem(slots_per_epoch).unwrap_or(0);
            (epoch, slot_index)
        }
    }

    /// Returns the first slot of the given epoch.
    pub fn get_first_slot_in_epoch(&self, epoch: u64) -> u64 {
        let first_normal_epoch = self.first_normal_epoch();
        if epoch <= first_normal_epoch {
            2u64.saturating_pow(epoch as u32)
                .saturating_sub(1)
                .saturating_mul(MINIMUM_SLOTS_PER_EPOCH)
        } else {
            epoch
                .saturating_sub(first_normal_epoch)
                .saturating_mul(self.slots_per_epoch())
                .saturating_add(self.first_normal_slot())
        }
    }

    /// Returns the last slot of the given epoch.
    #[inline(always)]
    pub fn get_last_slot_in_epoch(&self, epoch: u64) -> u64 {
        self.get_first_slot_in_epoch(epoch)
            .saturating_add(self.get_slots_in_epoch(epoch))
            .saturating_sub(1)
    }
}

impl Sysvar for EpochSchedule {
    impl_sysvar_get!(EPOCH_SCHEDULE_ID, 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a test `EpochSchedule` buffer with the given fields.
    fn encode(
        slots_per_epoch: u64,
        leader_schedule_slot_offset: u64,
        warmup: bool,
        first_normal_epoch: u64,
        first_normal_slot: u64,
    ) -> [u8; EpochSchedule::LEN] {
        let mut bytes = [0u8; EpochSchedule::LEN];
        bytes[0..8].copy_from_slice(&slots_per_epoch.to_le_bytes());
        bytes[8..16].copy_from_slice(&leader_schedule_slot_offset.to_le_bytes());
        bytes[16] = warmup as u8;
        bytes[17..25].copy_from_slice(&first_normal_epoch.to_le_bytes());
        bytes[25..33].copy_from_slice(&first_normal_slot.to_le_bytes());
        bytes
    }

    /// Reference implementation of the `custom` constructor from
    /// `solana-epoch-schedule`, used to build realistic test vectors.
    fn custom_bytes(
        slots_per_epoch: u64,
        leader_schedule_slot_offset: u64,
        warmup: bool,
    ) -> [u8; EpochSchedule::LEN] {
        assert!(slots_per_epoch >= MINIMUM_SLOTS_PER_EPOCH);
        let (first_normal_epoch, first_normal_slot) = if warmup {
            let next_power_of_two = slots_per_epoch.next_power_of_two();
            let log2_slots_per_epoch = next_power_of_two
                .trailing_zeros()
                .saturating_sub(MINIMUM_SLOTS_PER_EPOCH.trailing_zeros());
            (
                u64::from(log2_slots_per_epoch),
                next_power_of_two.saturating_sub(MINIMUM_SLOTS_PER_EPOCH),
            )
        } else {
            (0, 0)
        };
        encode(
            slots_per_epoch,
            leader_schedule_slot_offset,
            warmup,
            first_normal_epoch,
            first_normal_slot,
        )
    }

    #[test]
    fn struct_layout_matches_bincode() {
        assert_eq!(EpochSchedule::LEN, 33);
        assert_eq!(size_of::<EpochSchedule>(), 33);
        assert_eq!(align_of::<EpochSchedule>(), 1);
    }

    #[test]
    fn from_bytes_rejects_short() {
        let too_short = [0u8; 32];
        assert!(EpochSchedule::from_bytes(&too_short).is_err());
    }

    #[test]
    fn accessors_read_fields_correctly() {
        let bytes = encode(432_000, 432_000, true, 14, 524_256);
        let sched = EpochSchedule::from_bytes(&bytes).unwrap();

        assert_eq!(sched.slots_per_epoch(), 432_000);
        assert_eq!(sched.leader_schedule_slot_offset(), 432_000);
        assert!(sched.warmup());
        assert_eq!(sched.first_normal_epoch(), 14);
        assert_eq!(sched.first_normal_slot(), 524_256);
    }

    #[test]
    fn warmup_false_reads_as_false() {
        let bytes = encode(0, 0, false, 0, 0);
        let sched = EpochSchedule::from_bytes(&bytes).unwrap();
        assert!(!sched.warmup());
    }

    #[test]
    fn get_slots_in_epoch_matches_reference_without_warmup() {
        // Without warmup: every epoch has `slots_per_epoch` slots.
        let bytes = custom_bytes(DEFAULT_SLOTS_PER_EPOCH, DEFAULT_SLOTS_PER_EPOCH, false);
        let sched = EpochSchedule::from_bytes(&bytes).unwrap();

        assert_eq!(sched.get_slots_in_epoch(0), DEFAULT_SLOTS_PER_EPOCH);
        assert_eq!(sched.get_slots_in_epoch(100), DEFAULT_SLOTS_PER_EPOCH);
        assert_eq!(sched.get_slots_in_epoch(u64::MAX), DEFAULT_SLOTS_PER_EPOCH);
    }

    #[test]
    fn get_slots_in_epoch_matches_reference_with_warmup() {
        // With warmup: epoch N (pre-normal) has `MINIMUM_SLOTS_PER_EPOCH << N`
        // slots.
        let bytes = custom_bytes(DEFAULT_SLOTS_PER_EPOCH, DEFAULT_SLOTS_PER_EPOCH, true);
        let sched = EpochSchedule::from_bytes(&bytes).unwrap();

        assert_eq!(sched.get_slots_in_epoch(0), MINIMUM_SLOTS_PER_EPOCH);
        assert_eq!(sched.get_slots_in_epoch(1), MINIMUM_SLOTS_PER_EPOCH * 2);
        assert_eq!(sched.get_slots_in_epoch(2), MINIMUM_SLOTS_PER_EPOCH * 4);
        assert_eq!(
            sched.get_slots_in_epoch(sched.first_normal_epoch()),
            DEFAULT_SLOTS_PER_EPOCH
        );
    }

    #[test]
    fn get_first_slot_in_epoch_without_warmup() {
        let bytes = custom_bytes(DEFAULT_SLOTS_PER_EPOCH, DEFAULT_SLOTS_PER_EPOCH, false);
        let sched = EpochSchedule::from_bytes(&bytes).unwrap();

        assert_eq!(sched.get_first_slot_in_epoch(0), 0);
        assert_eq!(sched.get_first_slot_in_epoch(1), DEFAULT_SLOTS_PER_EPOCH);
        assert_eq!(
            sched.get_first_slot_in_epoch(5),
            5 * DEFAULT_SLOTS_PER_EPOCH
        );
    }

    #[test]
    fn get_last_slot_in_epoch_without_warmup() {
        let bytes = custom_bytes(DEFAULT_SLOTS_PER_EPOCH, DEFAULT_SLOTS_PER_EPOCH, false);
        let sched = EpochSchedule::from_bytes(&bytes).unwrap();

        // Last slot = first_slot + slots_in_epoch - 1.
        assert_eq!(sched.get_last_slot_in_epoch(0), DEFAULT_SLOTS_PER_EPOCH - 1);
        assert_eq!(
            sched.get_last_slot_in_epoch(1),
            2 * DEFAULT_SLOTS_PER_EPOCH - 1
        );
    }

    #[test]
    fn get_epoch_round_trips_through_first_slot() {
        // For any epoch in the normal range, `get_epoch(get_first_slot_in_epoch(e)) ==
        // e`.
        let bytes = custom_bytes(DEFAULT_SLOTS_PER_EPOCH, DEFAULT_SLOTS_PER_EPOCH, true);
        let sched = EpochSchedule::from_bytes(&bytes).unwrap();

        for epoch in [
            sched.first_normal_epoch(),
            sched.first_normal_epoch() + 1,
            sched.first_normal_epoch() + 100,
        ] {
            let first = sched.get_first_slot_in_epoch(epoch);
            assert_eq!(sched.get_epoch(first), epoch);
            let (e, idx) = sched.get_epoch_and_slot_index(first);
            assert_eq!(e, epoch);
            assert_eq!(idx, 0);
        }
    }

    #[test]
    fn get_leader_schedule_epoch_normal_range() {
        // Post-warmup, the leader schedule epoch is
        // `first_normal_epoch + (slot - first_normal_slot + offset) /
        // slots_per_epoch`.
        let bytes = custom_bytes(DEFAULT_SLOTS_PER_EPOCH, DEFAULT_SLOTS_PER_EPOCH, false);
        let sched = EpochSchedule::from_bytes(&bytes).unwrap();

        // Slot 0 with offset of one full epoch → leader schedule for
        // epoch 1.
        assert_eq!(sched.get_leader_schedule_epoch(0), 1);
        // Deep in epoch 5 + offset of one epoch → leader schedule for
        // epoch 6.
        assert_eq!(
            sched.get_leader_schedule_epoch(5 * DEFAULT_SLOTS_PER_EPOCH + 100),
            6
        );
    }

    #[test]
    fn get_leader_schedule_epoch_warmup_range() {
        // During warmup, the leader schedule epoch is `get_epoch(slot) + 1`.
        let bytes = custom_bytes(DEFAULT_SLOTS_PER_EPOCH, DEFAULT_SLOTS_PER_EPOCH, true);
        let sched = EpochSchedule::from_bytes(&bytes).unwrap();

        // Slot 0 during warmup is in epoch 0 → leader schedule epoch 1.
        assert_eq!(sched.get_leader_schedule_epoch(0), 1);
    }
}
