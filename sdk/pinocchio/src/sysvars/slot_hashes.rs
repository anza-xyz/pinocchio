//! Efficient, zero-copy access to SlotHashes sysvar data.

use crate::{
    account_info::{AccountInfo, Ref},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::clock::Slot,
};
use core::{mem, ops::Deref};

/// SysvarS1otHashes111111111111111111111111111
pub const SLOTHASHES_ID: Pubkey = [
    6, 167, 213, 23, 25, 47, 10, 175, 198, 242, 101, 227, 251, 119, 204, 122, 218, 130, 197, 41,
    208, 190, 59, 19, 110, 45, 0, 85, 32, 0, 0, 0,
];
/// Number of bytes in a hash.
pub const HASH_BYTES: usize = 32;
/// Sysvar data is:
/// len    (8 bytes): little-endian entry count (≤ 512)
/// entries(len × 40 bytes):    consecutive `(u64 slot, [u8;32] hash)` pairs
/// Size of the entry count field at the beginning of sysvar data.
pub const NUM_ENTRIES_SIZE: usize = mem::size_of::<u64>();
/// Size of a slot number in bytes.
pub const SLOT_SIZE: usize = mem::size_of::<Slot>();
/// Size of a single slot hash entry (slot + hash).
pub const ENTRY_SIZE: usize = SLOT_SIZE + HASH_BYTES;
/// Maximum number of slot hash entries that can be stored in the sysvar.
pub const MAX_ENTRIES: usize = 512;
/// Max size of the sysvar data in bytes. 20488. Golden on mainnet (never smaller)
pub const MAX_SIZE: usize = NUM_ENTRIES_SIZE + MAX_ENTRIES * ENTRY_SIZE;

/// A single entry in the `SlotHashes` sysvar.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C)]
pub struct SlotHashEntry {
    /// The slot number stored as little-endian bytes.
    slot_le: [u8; 8],
    /// The hash corresponding to the slot.
    pub hash: [u8; HASH_BYTES],
}

const _: [(); 1] = [(); mem::align_of::<SlotHashEntry>()];

/// SlotHashes provides read-only, zero-copy access to SlotHashes sysvar bytes.
pub struct SlotHashes<T: Deref<Target = [u8]>> {
    data: T,
    len: usize,
}

/// Reads the entry count from the first 8 bytes of data.
/// Returns None if the data is too short.
#[inline(always)]
fn read_entry_count_from_bytes(data: &[u8]) -> Option<usize> {
    if data.len() < NUM_ENTRIES_SIZE {
        return None;
    }
    Some(unsafe { u64::from_le_bytes(*(data.as_ptr() as *const [u8; 8])) } as usize)
}

/// Reads the entry count from the first 8 bytes of data.
///
/// # Safety
/// Caller must ensure data has at least NUM_ENTRIES_SIZE bytes.
#[inline(always)]
unsafe fn read_entry_count_from_bytes_unchecked(data: &[u8]) -> usize {
    u64::from_le_bytes(*(data.as_ptr() as *const [u8; 8])) as usize
}

/// Validates core SlotHashes constraints: entry count and buffer size requirements.
///
/// # Arguments
/// * `buffer_len` - Total buffer length including 8-byte header
/// * `declared_entries` - Optional declared entry count from header (None to skip this check)
///
/// # Returns
/// The maximum entries that fit in the buffer, or error if constraints violated
fn validate_slothashes_constraints(
    buffer_len: usize,
    declared_entries: Option<usize>,
) -> Result<usize, ProgramError> {
    // Must have space for 8-byte header
    if buffer_len < NUM_ENTRIES_SIZE {
        return Err(ProgramError::AccountDataTooSmall);
    }

    // Calculate how many entries can fit
    let data_len = buffer_len - NUM_ENTRIES_SIZE;
    if data_len % ENTRY_SIZE != 0 {
        return Err(ProgramError::InvalidArgument);
    }

    let max_entries = data_len / ENTRY_SIZE;
    if max_entries > MAX_ENTRIES {
        return Err(ProgramError::InvalidArgument);
    }

    if let Some(declared) = declared_entries {
        if declared > max_entries {
            return Err(ProgramError::InvalidArgument);
        }
        return Ok(declared);
    }

    Ok(max_entries)
}

/// Validates SlotHashes data format and returns the entry count.
fn parse_and_validate_data(data: &[u8]) -> Result<usize, ProgramError> {
    // Need at least the 8-byte length prefix.
    if data.len() < NUM_ENTRIES_SIZE {
        return Err(ProgramError::AccountDataTooSmall);
    }

    let num_entries = unsafe { read_entry_count_from_bytes_unchecked(data) };

    validate_slothashes_constraints(data.len(), Some(num_entries))
}

impl SlotHashEntry {
    /// Returns the slot number as a u64.
    #[inline(always)]
    pub fn slot(&self) -> Slot {
        u64::from_le_bytes(self.slot_le)
    }
}

impl<T: Deref<Target = [u8]>> SlotHashes<T> {
    /// Validates that a buffer is properly sized for SlotHashes data.
    ///
    /// Checks that the buffer length is 8 + (N * 40) for some N ≤ 512.
    #[inline]
    fn validate_buffer_size(buffer_len: usize) -> Result<(), ProgramError> {
        validate_slothashes_constraints(buffer_len, None)?;
        Ok(())
    }

    /// Creates a `SlotHashes` instance from arbitrary data with full validation.
    ///
    /// This constructor performs comprehensive validation of the data format
    /// including length prefix, entry count bounds, and buffer size requirements.
    /// Does not validate that entries are sorted in descending order.
    pub fn new(data: T) -> Result<Self, ProgramError> {
        let num_entries = parse_and_validate_data(&data)?;
        Ok(unsafe { Self::new_unchecked(data, num_entries) })
    }

    /// Creates a `SlotHashes` instance directly from a data container and entry count.
    /// Important: provide a valid len. Whether or not len is assumed to be
    /// the constant 20_488 (512 entries) is up to caller.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check the validity of the data or count.
    /// The caller must ensure:
    /// 1. The underlying byte slice in `data` represents valid SlotHashes data
    ///    (length prefix + entries, where entries are sorted in descending order by slot).
    /// 2. `len` is the correct number of entries (≤ MAX_ENTRIES), matching the prefix.
    /// 3. The data slice contains at least `NUM_ENTRIES_SIZE + len * ENTRY_SIZE` bytes.
    ///
    #[inline]
    pub unsafe fn new_unchecked(data: T, len: usize) -> Self {
        debug_assert!(len <= MAX_ENTRIES && data.len() >= NUM_ENTRIES_SIZE + len * ENTRY_SIZE);
        SlotHashes { data, len }
    }

    /// Gets the number of entries stored in this SlotHashes instance.
    /// Performs validation checks and returns the entry count if valid.
    pub fn get_entry_count(&self) -> Result<usize, ProgramError> {
        let data_entry_count = read_entry_count_from_bytes(&self.data).unwrap_or(0);
        if data_entry_count != self.len {
            return Err(ProgramError::InvalidArgument);
        }
        Ok(self.len)
    }

    /// Reads the entry count directly from the beginning of this SlotHashes instance **without validation**.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it performs no checks on the underlying data.
    /// The caller **must** ensure that:
    /// 1. The underlying data contains at least `NUM_ENTRIES_SIZE` (8) bytes.
    /// 2. The first 8 bytes represent a valid `u64` in little-endian format.
    /// 3. Calling this function without ensuring the above may lead to panics
    ///    (out-of-bounds access) or incorrect results.
    #[inline(always)]
    pub unsafe fn get_entry_count_unchecked(&self) -> usize {
        read_entry_count_from_bytes_unchecked(&self.data)
    }

    /// Validates offset parameters for fetching SlotHashes data.
    ///
    /// # Arguments
    /// * `offset` - Byte offset within the sysvar data
    /// * `buffer_len` - Length of the buffer that will receive the data
    ///
    /// # Returns
    /// Ok(()) if the offset is valid, Err otherwise
    fn validate_fetch_offset(offset: u64, buffer_len: usize) -> Result<(), ProgramError> {
        if offset >= MAX_SIZE as u64 {
            return Err(ProgramError::InvalidArgument);
        }
        if offset != 0
            && (offset < NUM_ENTRIES_SIZE as u64
                || (offset - NUM_ENTRIES_SIZE as u64) % ENTRY_SIZE as u64 != 0)
        {
            return Err(ProgramError::InvalidArgument);
        }
        if offset.saturating_add(buffer_len as u64) > MAX_SIZE as u64 {
            return Err(ProgramError::InvalidArgument);
        }

        Ok(())
    }

    /// Fetches the SlotHashes sysvar data directly via syscall into a provided buffer.
    ///
    /// # Arguments
    /// * `buffer` - A mutable slice to store the sysvar data. Must be at least 8 bytes
    ///   and the length must be 8 + (N * 40) for some N ≤ 512.
    /// * `offset` - Byte offset within the sysvar data to start fetching from.
    ///   Must be 0 (start of data) or 8 + N*40 (start of entry N) for valid alignment.
    ///   Note: SlotHashes data starts with an 8-byte length prefix followed by entries.
    ///
    /// # Returns
    /// The actual number of entries found in the sysvar data.
    ///
    /// For most use cases, prefer `from_account_info()` which provides zero-copy access.
    pub fn fetch_into(buffer: &mut [u8], offset: u64) -> Result<usize, ProgramError> {
        if buffer.len() != MAX_SIZE {
            Self::validate_buffer_size(buffer.len())?;
        }

        Self::validate_fetch_offset(offset, buffer.len())?;

        Self::fetch_into_unchecked(buffer, offset)?;

        let num_entries = read_entry_count_from_bytes(buffer).unwrap_or(0);

        // Reject oversized entry counts to prevent surprises
        if num_entries > MAX_ENTRIES {
            return Err(ProgramError::InvalidArgument);
        }

        let required_len = NUM_ENTRIES_SIZE + num_entries * ENTRY_SIZE;
        if buffer.len() < required_len {
            return Err(ProgramError::InvalidArgument);
        }

        Ok(num_entries)
    }

    /// Fetches the SlotHashes sysvar data directly via syscall into a provided buffer
    /// without validation.
    ///
    /// This method is for programs that cannot include the sysvar account
    /// but still need access to the slot hashes data.
    ///
    /// # Arguments
    /// * `buffer` - A mutable slice to store the sysvar data. The buffer length
    ///   determines how much data is fetched. Use 20,488 bytes for full data
    ///   on mainnet.
    /// * `offset` - Byte offset within the sysvar data to start fetching from.
    ///   Note: SlotHashes data starts with an 8-byte length prefix followed by entries.
    ///   Must be 0 (start of data) or 8 + N*40 (start of entry N) for valid alignment,
    ///   but this is not checked.
    ///
    /// # Returns
    /// Nothing - the caller constructs the SlotHashes view afterwards.
    ///
    /// For most use cases, prefer `from_account_info()` which provides zero-copy access.
    pub fn fetch_into_unchecked(buffer: &mut [u8], offset: u64) -> Result<(), ProgramError> {
        // Fetch sysvar data into caller-provided buffer.
        let result = unsafe {
            crate::syscalls::sol_get_sysvar(
                SLOTHASHES_ID.as_ptr(),
                buffer.as_mut_ptr(),
                offset,
                buffer.len() as u64, // length
            )
        };

        if result != 0 {
            return Err(ProgramError::InvalidArgument);
        }

        Ok(())
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

    /// Returns the entire slice of entries. Call once and reuse the slice if you
    /// need many look-ups.
    #[inline(always)]
    pub fn entries(&self) -> &[SlotHashEntry] {
        self.as_entries_slice()
    }

    /// Gets a reference to the entry at `index` or `None` if out of bounds.
    #[inline(always)]
    pub fn get_entry(&self, index: usize) -> Option<&SlotHashEntry> {
        self.as_entries_slice().get(index)
    }

    /// Finds the hash for a specific slot using binary search.
    ///
    /// Returns the hash if the slot is found, or `None` if not found.
    /// Assumes entries are sorted by slot in descending order.
    /// If calling repeatedly, prefer getting `entries()` in caller
    /// to avoid repeated slice construction.
    pub fn get_hash(&self, target_slot: Slot) -> Option<&[u8; HASH_BYTES]> {
        let entries = self.as_entries_slice();
        entries
            .binary_search_by(|probe_entry| probe_entry.slot().cmp(&target_slot).reverse())
            .ok()
            .map(|index| &entries[index].hash)
    }

    /// Finds the position (index) of a specific slot using binary search.
    ///
    /// Returns the index if the slot is found, or `None` if not found.
    /// Assumes entries are sorted by slot in descending order.
    /// If calling repeatedly, prefer getting `entries()` in caller
    /// to avoid repeated slice construction.
    pub fn position(&self, target_slot: Slot) -> Option<usize> {
        let entries = self.as_entries_slice();
        entries
            .binary_search_by(|probe_entry| probe_entry.slot().cmp(&target_slot).reverse())
            .ok()
    }

    /// Returns a `&[SlotHashEntry]` view into the underlying data.
    ///
    /// The constructor (in the safe path that called `parse_and_validate_data`)
    /// or caller (if unsafe `new_unchecked` path) is responsible for ensuring
    /// the slice is big enough and properly aligned.
    #[inline(always)]
    fn as_entries_slice(&self) -> &[SlotHashEntry] {
        if self.len == 0 {
            return &[];
        }

        debug_assert!(self.data.len() >= NUM_ENTRIES_SIZE + self.len * ENTRY_SIZE);

        let entries_ptr =
            unsafe { self.data.as_ptr().add(NUM_ENTRIES_SIZE) as *const SlotHashEntry };
        unsafe { core::slice::from_raw_parts(entries_ptr, self.len) }
    }

    /// # Safety
    /// Caller must ensure `index < self.len()`.
    #[inline(always)]
    pub unsafe fn get_entry_unchecked(&self, index: usize) -> &SlotHashEntry {
        debug_assert!(index < self.len);
        let offset = NUM_ENTRIES_SIZE + index * ENTRY_SIZE;
        &*(self.data.as_ptr().add(offset) as *const SlotHashEntry)
    }
}

impl<'a, T: Deref<Target = [u8]>> IntoIterator for &'a SlotHashes<T> {
    type Item = &'a SlotHashEntry;
    type IntoIter = core::slice::Iter<'a, SlotHashEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_entries_slice().iter()
    }
}

impl<'a> SlotHashes<Ref<'a, [u8]>> {
    /// Creates a `SlotHashes` instance by safely borrowing data from an `AccountInfo`.
    ///
    /// This function verifies that:
    /// - The account key matches the `SLOTHASHES_ID`
    pub fn from_account_info(account_info: &'a AccountInfo) -> Result<Self, ProgramError> {
        if account_info.key() != &SLOTHASHES_ID {
            return Err(ProgramError::InvalidArgument);
        }

        let data_ref = account_info.try_borrow_data()?;

        // Since the account key matches SLOTHASHES_ID, we can trust the runtime
        // to have provided valid sysvar data
        let num_entries = unsafe { read_entry_count_from_bytes_unchecked(&data_ref) };

        Ok(unsafe { Self::new_unchecked(data_ref, num_entries) })
    }
}

#[cfg(feature = "std")]
impl SlotHashes<std::vec::Vec<u8>> {
    /// Fetches the SlotHashes sysvar data directly via syscall. This copies
    /// the full sysvar data (`MAX_SIZE` bytes).
    pub fn fetch() -> Result<Self, ProgramError> {
        let mut data = std::vec![0u8; MAX_SIZE];

        // Use fetch_into to get the data and entry count
        let num_entries = Self::fetch_into(&mut data, 0)?;

        Ok(unsafe { Self::new_unchecked(data, num_entries) })
    }
}

#[cfg(test)]
#[path = "tests/slot_hashes_tests.rs"]
mod slot_hashes_tests;
