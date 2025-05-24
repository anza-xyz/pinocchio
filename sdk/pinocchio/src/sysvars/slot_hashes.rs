//! Efficient, zero-copy access to SlotHashes sysvar data.

use crate::{
    account_info::{AccountInfo, Ref},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::clock::Slot,
};
use core::{mem, ops::Deref, ptr};

/// SysvarS1otHashes111111111111111111111111111
pub const SLOTHASHES_ID: Pubkey = [
    6, 167, 213, 23, 25, 47, 10, 175, 198, 242, 101, 227, 251, 119, 204, 122, 218, 130, 197, 41,
    208, 190, 59, 19, 110, 45, 0, 85, 32, 0, 0, 0,
];
pub const MAX_ENTRIES: usize = 512;
pub const HASH_BYTES: usize = 32;

/// Sysvar data is:
/// len    (8 bytes): little-endian entry count (≤ 512)
/// entries(len × 40 bytes):    consecutive `(u64 slot, [u8;32] hash)` pairs
pub const NUM_ENTRIES_SIZE: usize = mem::size_of::<u64>();
pub const SLOT_SIZE: usize = mem::size_of::<Slot>();
pub const ENTRY_SIZE: usize = SLOT_SIZE + HASH_BYTES;

/// A single entry in the `SlotHashes` sysvar.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C)]
pub struct SlotHashEntry {
    /// The slot number.
    pub slot: Slot,
    /// The hash corresponding to the slot.
    pub hash: [u8; HASH_BYTES],
}

/// Provides zero-copy access to the data of the `SlotHashes` sysvar.
///
/// Internally it keeps either a plain slice `&'a [u8]` *or* the `Ref<'a, [u8]>`
/// returned by `AccountInfo::try_borrow_data()`.  Holding the `Ref` variant is
/// important because dropping the `Ref` would release the runtime borrow while
/// users still hold `&[u8]` references obtained from the struct.
enum Data<'a> {
    Slice(&'a [u8]),
    Ref(Ref<'a, [u8]>),
}

impl core::ops::Deref for Data<'_> {
    type Target = [u8];
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        match self {
            Data::Slice(s) => s,
            Data::Ref(r) => r,
        }
    }
}

/// SlotHashes provides read-only, zero-copy access to SlotHashes sysvar bytes.
pub struct SlotHashes<'a> {
    data: Data<'a>,
    len: usize,
}

impl<'a> SlotHashes<'a> {
    /// Creates a `SlotHashes` instance directly from a data container and entry count.
    /// Important: provide a valid len. Whether or not len is assumed to be
    /// the constant 20_488 (512 entries) is up to caller.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check the validity of the data or count.
    /// The caller must ensure:
    /// 1. The underlying byte slice in `data` represents valid SlotHashes data
    ///    (length prefix + entries).
    /// 2. `len` is the correct number of entries (≤ MAX_ENTRIES), matching the prefix.
    /// 3. The data slice contains at least `NUM_ENTRIES_SIZE + len * ENTRY_SIZE` bytes.
    /// 4. If `T` is `&[u8]`, that borrow rules are upheld.
    /// 5. Alignment is correct.
    ///
    #[inline(always)]
    pub unsafe fn new_unchecked_slice(data: &'a [u8], len: usize) -> Self {
        SlotHashes {
            data: Data::Slice(data),
            len,
        }
    }

    /// Same as `new_unchecked_slice` but keeps the `Ref` so the runtime borrow
    /// is held for the lifetime of the `SlotHashes` instance.
    ///
    /// # Safety
    ///
    /// Items 2 and 4 are normally auto-satisfied in Solana program contexts.
    ///
    /// 1. `data` must point to a byte slice that represents **valid** SlotHashes
    ///    contents for exactly `len` entries (i.e. it was previously validated, or
    ///    the caller otherwise guarantees correctness).
    /// 2. The memory backing `data` must remain valid for the entire lifetime `'a`
    ///    of the returned `SlotHashes` value.
    /// 3. The pointer in `data` must be correctly aligned for `SlotHashEntry` so
    ///    that later reference casts are sound.
    /// 4. Because a [`Ref`] is handed in, the caller must ensure the runtime
    ///    borrow rules are respected (no mutable aliasing etc.) for as long as
    ///    the returned `SlotHashes` exists.
    #[inline(always)]
    pub unsafe fn new_unchecked_ref(data: Ref<'a, [u8]>, len: usize) -> Self {
        SlotHashes {
            data: Data::Ref(data),
            len,
        }
    }

    /// Parses the length prefix of a SlotHashes account and validates that the
    /// slice is large enough for that many entries.  Only used by the *checked*
    /// construction paths; unchecked helpers are free to skip this work.
    ///
    /// Returns the number of entries on success.
    #[inline(always)]
    fn parse_and_validate_data(data: &[u8]) -> Result<usize, ProgramError> {
        // Need at least the 8-byte length prefix.
        if data.len() < NUM_ENTRIES_SIZE {
            return Err(ProgramError::AccountDataTooSmall);
        }
        let num_entries =
            unsafe { ptr::read_unaligned(data.as_ptr() as *const u64) }.to_le() as usize;

        // Reject oversized accounts so callers are not
        // surprised by silently truncated results.
        if num_entries > MAX_ENTRIES {
            return Err(ProgramError::InvalidAccountData);
        }

        let required_len = NUM_ENTRIES_SIZE + num_entries * ENTRY_SIZE;
        if data.len() < required_len {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(num_entries)
    }

    /// Gets the number of entries stored in the provided data slice.
    /// Performs validation checks and returns the entry count if valid.
    #[inline(always)]
    pub fn get_entry_count(data: &[u8]) -> Result<usize, ProgramError> {
        let num_entries = Self::parse_and_validate_data(data)?;
        Ok(num_entries)
    }

    /// Reads the entry count directly from the beginning of a byte slice **without validation**.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it performs no checks on the input slice.
    /// The caller **must** ensure that:
    /// 1. `data` contains at least `NUM_ENTRIES_SIZE` (8) bytes.
    /// 2. The first 8 bytes represent a valid `u64` in little-endian format.
    /// 3. Calling this function without ensuring the above may lead to panics
    ///    (out-of-bounds access) or incorrect results.
    #[inline(always)]
    pub unsafe fn get_entry_count_unchecked(data: &[u8]) -> usize {
        ptr::read_unaligned(data.as_ptr() as *const u64).to_le() as usize
    }

    /// Returns the number of `SlotHashEntry` items accessible.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if there are no entries.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Gets a reference to the `SlotHashEntry` at the specified index.
    /// Returns `None` if the index is out of bounds.
    #[inline(always)]
    pub fn get_entry(&self, index: usize) -> Option<&SlotHashEntry> {
        self.as_entries_slice().get(index)
    }

    /// Gets a reference without bounds checking.
    ///
    /// # Safety
    /// Caller must ensure `index < self.len()`.
    #[inline(always)]
    pub unsafe fn get_entry_unchecked(&self, index: usize) -> &SlotHashEntry {
        debug_assert!(index < self.len);
        let offset = NUM_ENTRIES_SIZE + index * ENTRY_SIZE;
        &*(self.data.deref().as_ptr().add(offset) as *const SlotHashEntry)
    }

    /// Finds the hash for a specific slot using binary search.
    ///
    /// Returns the hash if the slot is found, or `None` if not found.
    /// Assumes entries are sorted by slot in descending order.
    #[inline(always)]
    pub fn get_hash(&self, target_slot: Slot) -> Option<&[u8; HASH_BYTES]> {
        let entries = self.as_entries_slice();
        entries
            .binary_search_by(|probe_entry| probe_entry.slot.cmp(&target_slot).reverse())
            .ok()
            .map(|index| &entries[index].hash)
    }

    /// Finds the position (index) of a specific slot using binary search.
    ///
    /// Returns the index if the slot is found, or `None` if not found.
    /// Assumes entries are sorted by slot in descending order.
    #[inline(always)]
    pub fn position(&self, target_slot: Slot) -> Option<usize> {
        let entries = self.as_entries_slice();
        entries
            .binary_search_by(|probe_entry| probe_entry.slot.cmp(&target_slot).reverse())
            .ok()
    }

    /// Returns a `&[SlotHashEntry]` view into the underlying data.
    ///
    /// The constructor (either the safe path that called `parse_and_validate_data` or
    /// the unsafe `new_unchecked`) is responsible for ensuring the slice is big enough
    /// and properly aligned.
    #[inline(always)]
    fn as_entries_slice(&self) -> &[SlotHashEntry] {
        if self.len == 0 {
            return &[];
        }

        debug_assert!(self.data.deref().len() >= NUM_ENTRIES_SIZE + self.len * ENTRY_SIZE);

        let entries_ptr =
            unsafe { self.data.deref().as_ptr().add(NUM_ENTRIES_SIZE) as *const SlotHashEntry };
        unsafe { core::slice::from_raw_parts(entries_ptr, self.len) }
    }
}

// Implementation block specific to the safe Ref version
impl<'a> SlotHashes<'a> {
    /// Creates a `SlotHashes` instance by safely borrowing data from an `AccountInfo`.
    ///
    /// This function verifies that:
    /// - The account key matches the `SLOTHASHES_ID`
    /// - The data contains a valid length prefix
    /// - The data is sufficiently large to hold the indicated number of entries
    #[inline(always)]
    pub fn from_account_info(account_info: &'a AccountInfo) -> Result<Self, ProgramError> {
        if account_info.key() != &SLOTHASHES_ID {
            return Err(ProgramError::InvalidArgument);
        }

        let data_ref = account_info.try_borrow_data()?;
        // Ensure the byte slice is suitably aligned for `SlotHashEntry`
        if (data_ref.as_ptr() as usize) % mem::align_of::<SlotHashEntry>() != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        let num_entries = Self::parse_and_validate_data(&data_ref)?;

        Ok(unsafe { Self::new_unchecked_ref(data_ref, num_entries) })
    }
}

/// Reads the entry count directly from the beginning of a byte slice **without validation**.
/// (This is identical to the struct method get_entry_count_unchecked,
/// added here for discoverability with other unchecked fns)
///
/// # Safety
///
/// This function is unsafe because it performs no checks on the input slice.
/// The caller **must** ensure that:
/// 1. `data` contains at least `NUM_ENTRIES_SIZE` (8) bytes.
/// 2. The first 8 bytes represent a valid `u64` in little-endian format.
/// 3. Calling this function without ensuring the above may lead to panics
///    (out-of-bounds access) or incorrect results.
#[inline(always)]
pub unsafe fn get_entry_count_from_slice_unchecked(data: &[u8]) -> usize {
    ptr::read_unaligned(data.as_ptr() as *const u64).to_le() as usize
}

/// Performs an **unsafe** naive binary search directly on a raw byte slice.
///
/// # Safety
/// Caller must guarantee `data` contains a valid `SlotHashes` structure and that
/// `num_entries` is the correct count of entries in `data`. It is up to caller whether
/// to use MAX_ENTRIES or to use a call such as `get_entry_count_from_slice_unchecked`
#[inline(always)]
pub unsafe fn position_from_slice_binary_search_unchecked(
    data: &[u8],
    target_slot: Slot,
    num_entries: usize,
) -> Option<usize> {
    // caller promises `data` is large enough and properly formatted
    SlotHashes::new_unchecked_slice(data, num_entries).position(target_slot)
}

/// Gets a reference to the hash for a specific slot from a raw byte slice **without validation**.
///
/// # Safety
/// Caller must guarantee `data` contains a valid `SlotHashes` structure.
/// Caller must guarantee `num_entries` is the correct count of entries in `data`. It is up
/// to caller whether to use MAX_ENTRIES or to use a call such as `get_entry_count_from_slice_unchecked`
#[inline(always)]
pub unsafe fn get_hash_from_slice_unchecked(
    data: &[u8],
    target_slot: Slot,
    num_entries: usize,
) -> Option<&[u8; HASH_BYTES]> {
    let index = position_from_slice_binary_search_unchecked(data, target_slot, num_entries)?;
    let hash_offset = NUM_ENTRIES_SIZE + index * ENTRY_SIZE + SLOT_SIZE;
    Some(&*(data.as_ptr().add(hash_offset) as *const [u8; HASH_BYTES]))
}

/// Gets a reference to the `SlotHashEntry` at a specific index from a raw byte slice **without validation**.
///
/// # Safety
/// Caller must guarantee `data` contains a valid `SlotHashes` structure and that `index` is less than the entry count derived from the data's prefix.
#[inline(always)]
pub unsafe fn get_entry_from_slice_unchecked(data: &[u8], index: usize) -> &SlotHashEntry {
    let entry_offset = NUM_ENTRIES_SIZE + index * ENTRY_SIZE;
    let entry_bytes = data.get_unchecked(entry_offset..(entry_offset + ENTRY_SIZE));
    &*(entry_bytes.as_ptr() as *const SlotHashEntry)
}

impl<'s> IntoIterator for &'s SlotHashes<'s> {
    type Item = &'s SlotHashEntry;
    type IntoIter = core::slice::Iter<'s, SlotHashEntry>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.as_entries_slice().iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{align_of, size_of};
    extern crate std;
    #[allow(unused_imports)]
    use std::vec::Vec;

    #[test]
    fn test_layout_constants() {
        assert_eq!(NUM_ENTRIES_SIZE, size_of::<u64>());
        assert_eq!(SLOT_SIZE, size_of::<u64>());
        assert_eq!(HASH_BYTES, 32);
        assert_eq!(ENTRY_SIZE, size_of::<u64>() + 32);
        assert_eq!(size_of::<SlotHashEntry>(), ENTRY_SIZE);
        assert_eq!(align_of::<SlotHashEntry>(), align_of::<u64>());
        assert_eq!(
            SLOTHASHES_ID,
            [
                6, 167, 213, 23, 25, 47, 10, 175, 198, 242, 101, 227, 251, 119, 204, 122, 218, 130,
                197, 41, 208, 190, 59, 19, 110, 45, 0, 85, 32, 0, 0, 0,
            ]
        );
        const BASE_58: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        // quick base58 comparison just for test
        pub fn check_base58(input_bytes: &[u8], expected_b58: &str) {
            let mut b58_digits_rev = std::vec![0u8];
            for &byte_val in input_bytes {
                let mut carry = byte_val as u32;
                for digit_ref in b58_digits_rev.iter_mut() {
                    let temp_val = ((*digit_ref as u32) << 8) | carry;
                    *digit_ref = (temp_val % 58) as u8;
                    carry = temp_val / 58;
                }
                while carry > 0 {
                    b58_digits_rev.push((carry % 58) as u8);
                    carry /= 58;
                }
            }
            for &byte_val in input_bytes {
                if byte_val == 0 {
                    b58_digits_rev.push(0)
                } else {
                    break;
                }
            }
            let mut output_chars = Vec::with_capacity(b58_digits_rev.len());
            for &digit_val in b58_digits_rev.iter().rev() {
                output_chars.push(BASE_58[digit_val as usize]);
            }
            assert_eq!(expected_b58.as_bytes(), output_chars.as_slice());
        }
        check_base58(
            &SLOTHASHES_ID,
            "SysvarS1otHashes111111111111111111111111111",
        );
    }

    fn create_mock_data(entries: &[(u64, [u8; 32])]) -> Vec<u8> {
        let num_entries = entries.len() as u64;
        let data_len = NUM_ENTRIES_SIZE + entries.len() * ENTRY_SIZE;
        let mut data = std::vec![0u8; data_len];
        data[0..NUM_ENTRIES_SIZE].copy_from_slice(&num_entries.to_le_bytes());
        let mut offset = NUM_ENTRIES_SIZE;
        for (slot, hash) in entries {
            data[offset..offset + SLOT_SIZE].copy_from_slice(&slot.to_le_bytes());
            data[offset + SLOT_SIZE..offset + ENTRY_SIZE].copy_from_slice(hash);
            offset += ENTRY_SIZE;
        }
        data
    }

    fn generate_mock_entries(
        num_entries: usize,
        start_slot: u64,
        strategy: DecrementStrategy,
    ) -> Vec<(u64, [u8; 32])> {
        let mut entries = Vec::with_capacity(num_entries);
        let mut current_slot = start_slot;
        for i in 0..num_entries {
            let hash_byte = (i % 256) as u8;
            let hash = [hash_byte; 32];
            entries.push((current_slot, hash));
            let random_val = simple_prng(i as u64);
            let decrement = match strategy {
                DecrementStrategy::Strictly1 => 1,
                DecrementStrategy::Average1_05 => {
                    if random_val % 20 == 0 {
                        2
                    } else {
                        1
                    }
                }
                #[allow(dead_code)] // May be used by benchmarks
                DecrementStrategy::Average2 => {
                    if random_val % 2 == 0 {
                        1
                    } else {
                        3
                    }
                }
            };
            current_slot = current_slot.saturating_sub(decrement);
        }
        entries
    }

    #[cfg(feature = "std")]
    mod std_tests {
        use super::*;

        #[test]
        fn test_iterator_into_ref() {
            let entries = generate_mock_entries(10, 50, DecrementStrategy::Strictly1);
            let data = create_mock_data(&entries);
            let sh = unsafe { SlotHashes::new_unchecked_slice(data.as_slice(), entries.len()) };

            let mut collected: Vec<u64> = Vec::new();
            for e in &sh {
                collected.push(e.slot);
            }
            let expected: Vec<u64> = entries.iter().map(|(s, _)| *s).collect();
            assert_eq!(collected, expected);

            let iter = (&sh).into_iter();
            assert_eq!(iter.len(), sh.len());
        }

        #[test]
        fn test_from_account_info_constructor() {
            // Cover the safe constructor that goes through `AccountInfo` and holds the Ref.
            use crate::account_info::{Account, AccountInfo};
            use crate::pubkey::Pubkey;
            use core::{mem, ptr};

            const NUM_ENTRIES: usize = 3;
            const START_SLOT: u64 = 1234;
            let mock_entries =
                generate_mock_entries(NUM_ENTRIES, START_SLOT, DecrementStrategy::Strictly1);
            let data = create_mock_data(&mock_entries);

            let mut aligned_backing: Vec<u64>;
            #[allow(unused_assignments)]
            let mut acct_ptr: *mut Account = core::ptr::null_mut();

            #[repr(C)]
            struct FakeAccount {
                borrow_state: u8,
                is_signer: u8,
                is_writable: u8,
                executable: u8,
                original_data_len: u32,
                key: Pubkey,
                owner: Pubkey,
                lamports: u64,
                data_len: u64,
            }

            unsafe {
                // 1) Build a contiguous Vec<u8> with header followed by SlotHashes payload.
                let header_size = mem::size_of::<FakeAccount>();
                let mut blob: Vec<u8> = std::vec![0u8; header_size + data.len()];

                let header_ptr = &mut blob[0] as *mut u8 as *mut FakeAccount;
                ptr::write(
                    header_ptr,
                    FakeAccount {
                        borrow_state: 0,
                        is_signer: 0,
                        is_writable: 0,
                        executable: 0,
                        original_data_len: 0,
                        key: SLOTHASHES_ID,
                        owner: [0u8; 32],
                        lamports: 0,
                        data_len: data.len() as u64,
                    },
                );

                ptr::copy_nonoverlapping(
                    data.as_ptr(),
                    blob.as_mut_ptr().add(header_size),
                    data.len(),
                );

                let word_len = (blob.len() + 7) / 8;
                aligned_backing = std::vec![0u64; word_len];
                ptr::copy_nonoverlapping(
                    blob.as_ptr(),
                    aligned_backing.as_mut_ptr() as *mut u8,
                    blob.len(),
                );

                // Purposely shadow the earlier variables so the remainder of the test
                // works unchanged.
                let ptr_u8 = aligned_backing.as_mut_ptr() as *mut u8;
                acct_ptr = ptr_u8 as *mut Account;
            }

            let account_info = AccountInfo { raw: acct_ptr };
            let slot_hashes = SlotHashes::from_account_info(&account_info)
                .expect("from_account_info should succeed with well-formed data");

            // Basic sanity checks on the returned view.
            assert_eq!(slot_hashes.len(), NUM_ENTRIES);
            for (i, entry) in slot_hashes.into_iter().enumerate() {
                assert_eq!(entry.slot, mock_entries[i].0);
                assert_eq!(entry.hash, mock_entries[i].1);
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    #[allow(dead_code)]
    enum DecrementStrategy {
        Strictly1,
        Average1_05,
        Average2,
    }

    // Stand-in for proper fuzz (todo)
    fn simple_prng(seed: u64) -> u64 {
        const A: u64 = 16807;
        const M: u64 = 2147483647;
        let initial_state = if seed == 0 { 1 } else { seed };
        (A.wrapping_mul(initial_state)) % M
    }

    #[test]
    fn test_binary_search_no_std() {
        const TEST_NUM_ENTRIES: usize = 512;
        const START_SLOT: u64 = 2000;

        // Generate entries using Avg1.05 strategy
        let entries =
            generate_mock_entries(TEST_NUM_ENTRIES, START_SLOT, DecrementStrategy::Average1_05);
        let data = create_mock_data(&entries);
        let entry_count = entries.len();

        // Get first, middle, and last generated slots for testing
        let first_slot = entries[0].0;
        let mid_index = entry_count / 2;
        let mid_slot = entries[mid_index].0;
        let last_slot = entries[entry_count - 1].0;

        // Create SlotHashes using the unsafe constructor with a slice
        let slot_hashes = unsafe { SlotHashes::new_unchecked_slice(data.as_slice(), entry_count) };

        // Test the default (interpolation) binary search algorithm
        assert_eq!(slot_hashes.position(first_slot), Some(0));

        let expected_mid_index = Some(mid_index);
        let actual_pos_mid = slot_hashes.position(mid_slot);

        // Extract surrounding entries for context in case of failure
        let start_idx = mid_index.saturating_sub(2);
        let end_idx = core::cmp::min(entry_count, mid_index.saturating_add(3));
        let surrounding_slots: Vec<_> = entries[start_idx..end_idx].iter().map(|e| e.0).collect();
        assert_eq!(
            actual_pos_mid, expected_mid_index,
            "position({}) failed! Surrounding slots: {:?}",
            mid_slot, surrounding_slots
        );

        assert_eq!(slot_hashes.position(last_slot), Some(entry_count - 1));

        // Test non-existent slots
        assert_eq!(slot_hashes.position(START_SLOT + 1), None); // Slot above start

        // Find an actual gap to test a guaranteed non-existent internal slot
        let mut missing_internal_slot = None;
        for i in 0..(entries.len() - 1) {
            if entries[i].0 > entries[i + 1].0 + 1 {
                missing_internal_slot = Some(entries[i + 1].0 + 1);
                break;
            }
        }
        assert!(
            missing_internal_slot.is_some(),
            "Test requires at least one gap between slots"
        );
        assert_eq!(slot_hashes.position(missing_internal_slot.unwrap()), None);

        // Test get_hash (interpolation)
        assert_eq!(slot_hashes.get_hash(first_slot), Some(&entries[0].1));
        assert_eq!(slot_hashes.get_hash(mid_slot), Some(&entries[mid_index].1));
        assert_eq!(
            slot_hashes.get_hash(last_slot),
            Some(&entries[entry_count - 1].1)
        );
        assert_eq!(slot_hashes.get_hash(START_SLOT + 1), None);

        // Test empty list explicitly
        let empty_entries = generate_mock_entries(0, START_SLOT, DecrementStrategy::Strictly1);
        let empty_data = create_mock_data(&empty_entries);
        let empty_hashes = unsafe { SlotHashes::new_unchecked_slice(empty_data.as_slice(), 0) };
        assert_eq!(empty_hashes.get_hash(100), None);

        let pos_start_plus_1 = slot_hashes.position(START_SLOT + 1);
        assert!(
            pos_start_plus_1.is_none(),
            "position(START_SLOT + 1) should be None"
        );
    }

    // No-std compatible tests
    #[test]
    fn test_basic_getters_and_iterator_no_std() {
        const NUM_ENTRIES: usize = 512;
        const START_SLOT: u64 = 2000;
        let entries = generate_mock_entries(NUM_ENTRIES, START_SLOT, DecrementStrategy::Strictly1);
        let data = create_mock_data(&entries);
        let slot_hashes = unsafe { SlotHashes::new_unchecked_slice(data.as_slice(), NUM_ENTRIES) };

        // Test len() and is_empty()
        assert_eq!(slot_hashes.len(), NUM_ENTRIES);
        assert!(!slot_hashes.is_empty());

        // Test get_entry()
        let entry0 = slot_hashes.get_entry(0);
        assert!(entry0.is_some());
        assert_eq!(entry0.unwrap().slot, START_SLOT); // Check against start slot
        assert_eq!(entry0.unwrap().hash, [0u8; HASH_BYTES]); // First generated hash is [0u8; 32]

        let entry2 = slot_hashes.get_entry(NUM_ENTRIES - 1); // Last entry
        assert!(entry2.is_some());
        // Check last entry against generated data
        assert_eq!(entry2.unwrap().slot, entries[NUM_ENTRIES - 1].0);
        assert_eq!(entry2.unwrap().hash, entries[NUM_ENTRIES - 1].1);
        assert!(slot_hashes.get_entry(NUM_ENTRIES).is_none()); // Out of bounds

        // Test iterator
        // Use enumerate to avoid clippy lint about indexing
        for (i, entry) in slot_hashes.into_iter().enumerate() {
            assert_eq!(entry.slot, entries[i].0);
            assert_eq!(entry.hash, entries[i].1);
        }
        // Check that the iterator is exhausted
        assert!(slot_hashes.into_iter().nth(NUM_ENTRIES).is_none());

        // Test ExactSizeIterator hint
        let mut iter_hint = slot_hashes.into_iter();
        assert_eq!(iter_hint.size_hint(), (NUM_ENTRIES, Some(NUM_ENTRIES)));
        iter_hint.next();
        assert_eq!(
            iter_hint.size_hint(),
            (NUM_ENTRIES - 1, Some(NUM_ENTRIES - 1))
        );
        // Skip to end
        for _ in 1..NUM_ENTRIES {
            iter_hint.next();
        }
        iter_hint.next();
        assert_eq!(iter_hint.size_hint(), (0, Some(0)));

        // Test empty case
        let empty_data = create_mock_data(&[]);
        let empty_hashes = unsafe { SlotHashes::new_unchecked_slice(empty_data.as_slice(), 0) };
        assert_eq!(empty_hashes.len(), 0);
        assert!(empty_hashes.is_empty());
        assert!(empty_hashes.get_entry(0).is_none());
        assert!(empty_hashes.into_iter().next().is_none());
    }

    #[test]
    fn test_get_entry_count_no_std() {
        // Valid data (2 entries)
        let entries: &[(Slot, [u8; HASH_BYTES])] =
            &[(100, [1u8; HASH_BYTES]), (98, [2u8; HASH_BYTES])];
        let num_entries_bytes = (entries.len() as u64).to_le_bytes();
        const TEST_LEN: usize = 2;
        let mut raw_data = [0u8; NUM_ENTRIES_SIZE + TEST_LEN * ENTRY_SIZE];
        raw_data[..NUM_ENTRIES_SIZE].copy_from_slice(&num_entries_bytes);
        let mut cursor = NUM_ENTRIES_SIZE;
        for (slot, hash) in entries {
            raw_data[cursor..cursor + SLOT_SIZE].copy_from_slice(&slot.to_le_bytes());
            cursor += SLOT_SIZE;
            raw_data[cursor..cursor + HASH_BYTES].copy_from_slice(hash.as_ref());
            cursor += HASH_BYTES;
        }
        let data_slice = &raw_data[..cursor];
        let count_res = SlotHashes::get_entry_count(data_slice);
        assert!(count_res.is_ok());
        assert_eq!(count_res.unwrap(), 2);
        let count_res_unchecked = unsafe { SlotHashes::get_entry_count_unchecked(data_slice) };
        assert_eq!(count_res_unchecked, 2);

        // Data too small (less than len prefix)
        let short_data_1 = &data_slice[0..NUM_ENTRIES_SIZE - 1];
        let res1 = SlotHashes::get_entry_count(short_data_1);
        assert!(matches!(res1, Err(ProgramError::AccountDataTooSmall)));

        // Data too small (correct len prefix, but not enough data for entries)
        let short_data_2 = &data_slice[0..NUM_ENTRIES_SIZE + ENTRY_SIZE]; // Only space for 1 entry
        let res2 = SlotHashes::get_entry_count(short_data_2);
        assert!(matches!(res2, Err(ProgramError::InvalidAccountData)));
        let count_res_unchecked_2 = unsafe { SlotHashes::get_entry_count_unchecked(short_data_2) };
        assert_eq!(count_res_unchecked_2, 2);

        // Empty data is valid
        let empty_num_bytes = (0u64).to_le_bytes();
        let mut empty_raw_data = [0u8; NUM_ENTRIES_SIZE];
        empty_raw_data[..NUM_ENTRIES_SIZE].copy_from_slice(&empty_num_bytes);
        let empty_res = SlotHashes::get_entry_count(empty_raw_data.as_slice());
        assert!(empty_res.is_ok());
        assert_eq!(empty_res.unwrap(), 0);
        let unsafe_empty_len =
            unsafe { SlotHashes::get_entry_count_unchecked(empty_raw_data.as_slice()) };
        assert_eq!(unsafe_empty_len, 0);
    }

    #[test]
    fn test_get_entry_unchecked_no_std() {
        let single_entry: &[(Slot, [u8; HASH_BYTES])] = &[(100, [1u8; HASH_BYTES])];
        let num_entries_bytes_1 = (single_entry.len() as u64).to_le_bytes();
        const TEST_LEN_1: usize = 1;
        let mut raw_data_1 = [0u8; NUM_ENTRIES_SIZE + TEST_LEN_1 * ENTRY_SIZE];
        raw_data_1[..NUM_ENTRIES_SIZE].copy_from_slice(&num_entries_bytes_1);
        raw_data_1[NUM_ENTRIES_SIZE..NUM_ENTRIES_SIZE + SLOT_SIZE]
            .copy_from_slice(&single_entry[0].0.to_le_bytes());
        raw_data_1[NUM_ENTRIES_SIZE + SLOT_SIZE..].copy_from_slice(single_entry[0].1.as_ref());
        let slot_hashes = unsafe { SlotHashes::new_unchecked_slice(&raw_data_1[..], 1) };

        // Safety: index 0 is valid because len is 1
        let entry = unsafe { slot_hashes.get_entry_unchecked(0) };
        assert_eq!(entry.slot, 100);
        assert_eq!(entry.hash, [1u8; HASH_BYTES]);
    }

    #[test]
    fn test_iterator_into_ref_no_std() {
        const NUM: usize = 16;
        const START: u64 = 100;
        let entries = generate_mock_entries(NUM, START, DecrementStrategy::Strictly1);
        let data = create_mock_data(&entries);
        let sh = unsafe { SlotHashes::new_unchecked_slice(data.as_slice(), NUM) };

        // Collect slots via iterator
        let mut sum: u64 = 0;
        for e in &sh {
            sum += e.slot;
        }
        let expected_sum: u64 = entries.iter().map(|(s, _)| *s).sum();
        assert_eq!(sum, expected_sum);

        let iter = (&sh).into_iter();
        assert_eq!(iter.len(), sh.len());
    }
}

#[cfg(test)]
mod edge_tests {
    use super::*;
    extern crate std;
    use crate::account_info::{Account, AccountInfo};
    use crate::pubkey::Pubkey;
    use core::{mem, ptr};
    use std::vec::Vec;

    fn raw_slot_hashes(declared_len: u64, entries: &[(u64, [u8; HASH_BYTES])]) -> Vec<u8> {
        let mut v = Vec::with_capacity(NUM_ENTRIES_SIZE + entries.len() * ENTRY_SIZE);
        v.extend_from_slice(&declared_len.to_le_bytes());
        for (slot, hash) in entries {
            v.extend_from_slice(&slot.to_le_bytes());
            v.extend_from_slice(hash);
        }
        v
    }

    unsafe fn account_info_with(key: Pubkey, data: &[u8]) -> AccountInfo {
        #[repr(C)]
        struct Header {
            borrow_state: u8,
            is_signer: u8,
            is_writable: u8,
            executable: u8,
            original_data_len: u32,
            key: Pubkey,
            owner: Pubkey,
            lamports: u64,
            data_len: u64,
        }
        let hdr_len = mem::size_of::<Header>();
        let mut backing = std::vec![0u8; hdr_len + data.len()];
        let hdr_ptr = backing.as_mut_ptr() as *mut Header;
        ptr::write(
            hdr_ptr,
            Header {
                borrow_state: 0,
                is_signer: 0,
                is_writable: 0,
                executable: 0,
                original_data_len: 0,
                key,
                owner: [0u8; 32],
                lamports: 0,
                data_len: data.len() as u64,
            },
        );
        ptr::copy_nonoverlapping(data.as_ptr(), backing.as_mut_ptr().add(hdr_len), data.len());
        // Leak backing so the slice outlives the AccountInfo for the duration of the test.
        core::mem::forget(backing);
        AccountInfo {
            raw: hdr_ptr as *mut Account,
        }
    }

    #[test]
    fn wrong_key_from_account_info() {
        let bytes = raw_slot_hashes(0, &[]);
        let acct = unsafe { account_info_with([1u8; 32], &bytes) };
        assert!(matches!(
            SlotHashes::from_account_info(&acct),
            Err(ProgramError::InvalidArgument)
        ));
    }

    #[test]
    fn too_many_entries_rejected() {
        let bytes = raw_slot_hashes((MAX_ENTRIES as u64) + 1, &[]);
        assert!(matches!(
            SlotHashes::get_entry_count(&bytes),
            Err(ProgramError::InvalidAccountData)
        ));
    }

    #[test]
    fn truncated_payload_rejected() {
        let entry = (123u64, [7u8; HASH_BYTES]);
        let bytes = raw_slot_hashes(2, &[entry]); // says 2 but provides 1
        assert!(matches!(
            SlotHashes::get_entry_count(&bytes),
            Err(ProgramError::InvalidAccountData)
        ));
    }

    #[test]
    fn duplicate_slots_binary_search_safe() {
        let entries = &[
            (200, [0u8; HASH_BYTES]),
            (200, [1u8; HASH_BYTES]),
            (199, [2u8; HASH_BYTES]),
        ];
        let bytes = raw_slot_hashes(entries.len() as u64, entries);
        let sh = unsafe { SlotHashes::new_unchecked_slice(&bytes, entries.len()) };
        let dup_pos = sh.position(200).expect("slot 200 must exist");
        assert!(
            dup_pos <= 1,
            "binary_search should return one of the duplicate indices (0 or 1)"
        );
        assert_eq!(sh.get_hash(199), Some(&entries[2].1));
    }

    #[test]
    fn zero_len_minimal_slice_iterates_empty() {
        let zero_bytes = 0u64.to_le_bytes();
        let sh = unsafe { SlotHashes::new_unchecked_slice(&zero_bytes, 0) };
        assert_eq!(sh.len(), 0);
        assert!(sh.into_iter().next().is_none());
    }
}
