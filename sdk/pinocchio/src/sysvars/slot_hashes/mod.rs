//! Efficient, zero-copy access to SlotHashes sysvar data.

pub mod raw;
#[doc(inline)]
pub use raw::{fetch_into, fetch_into_unchecked, validate_fetch_offset};

#[cfg(test)]
mod test;
#[cfg(test)]
mod test_edge;
#[cfg(test)]
mod test_raw;
#[cfg(test)]
mod test_utils;

use crate::{
    account_info::{AccountInfo, Ref},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::clock::Slot,
};
use core::{mem, ops::Deref, slice::from_raw_parts};
#[cfg(feature = "std")]
use std::boxed::Box;

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
/// A single hash.
pub type Hash = [u8; HASH_BYTES];

/// A single entry in the `SlotHashes` sysvar.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C)]
pub struct SlotHashEntry {
    /// The slot number stored as little-endian bytes.
    slot_le: [u8; 8],
    /// The hash corresponding to the slot.
    pub hash: Hash,
}

// Fail compilation if `SlotHashEntry` is not byte-aligned.
const _: [(); 1] = [(); mem::align_of::<SlotHashEntry>()];

/// SlotHashes provides read-only, zero-copy access to SlotHashes sysvar bytes.
pub struct SlotHashes<T: Deref<Target = [u8]>> {
    data: T,
}

/// Log a `Hash` from a program.
pub fn log(hash: &Hash) {
    crate::pubkey::log(hash);
}

/// Reads the entry count from the first 8 bytes of data.
/// Returns None if the data is too short.
#[inline(always)]
pub(crate) fn read_entry_count_from_bytes(data: &[u8]) -> Option<usize> {
    if data.len() < NUM_ENTRIES_SIZE {
        return None;
    }
    Some(unsafe {
        // SAFETY: `data` is guaranteed to be at least `NUM_ENTRIES_SIZE` bytes long by the
        // preceding length check, so it is sound to read the first 8 bytes and interpret
        // them as a little-endian `u64`.
        u64::from_le_bytes(*(data.as_ptr() as *const [u8; NUM_ENTRIES_SIZE]))
    } as usize)
}

/// Reads the entry count from the first 8 bytes of data.
///
/// # Safety
/// Caller must ensure data has at least NUM_ENTRIES_SIZE bytes.
#[inline(always)]
pub(crate) unsafe fn read_entry_count_from_bytes_unchecked(data: &[u8]) -> usize {
    u64::from_le_bytes(*(data.as_ptr() as *const [u8; NUM_ENTRIES_SIZE])) as usize
}

/// Validates SlotHashes data format.
///
/// The function checks:
/// 1. The buffer is large enough to contain the entry count.
/// 2. The buffer length is sufficient to hold the declared number of entries.
///
/// It returns `Ok(())` if the data is well-formed, otherwise an appropriate
/// `ProgramError` describing the issue.
#[inline]
fn parse_and_validate_data(data: &[u8]) -> Result<(), ProgramError> {
    if data.len() < NUM_ENTRIES_SIZE {
        return Err(ProgramError::AccountDataTooSmall);
    }

    // SAFETY: We've confirmed that data has enough bytes to read the entry count.
    let num_entries = unsafe { read_entry_count_from_bytes_unchecked(data) };

    let min_size = NUM_ENTRIES_SIZE + num_entries * ENTRY_SIZE;
    if data.len() < min_size {
        return Err(ProgramError::AccountDataTooSmall);
    }

    Ok(())
}

impl SlotHashEntry {
    /// Returns the slot number as a u64.
    #[inline(always)]
    pub fn slot(&self) -> Slot {
        u64::from_le_bytes(self.slot_le)
    }
}

impl<T: Deref<Target = [u8]>> SlotHashes<T> {
    /// Creates a `SlotHashes` instance with validation of the entry count and buffer size.
    ///
    /// This constructor validates that the buffer has at least enough bytes to contain
    /// the declared number of entries. The buffer can be any size >= the minimum required,
    /// making it suitable for both full MAX_SIZE buffers and smaller test data.
    /// Does not validate that entries are sorted in descending order.
    #[inline(always)]
    pub fn new(data: T) -> Result<Self, ProgramError> {
        parse_and_validate_data(&data)?;
        // SAFETY: `parse_and_validate_data` verifies that the data slice has at least
        // `NUM_ENTRIES_SIZE` bytes for the entry count and enough additional bytes to
        // contain the declared number of entries, thus upholding all invariants required
        // by `SlotHashes::new_unchecked`.
        Ok(unsafe { Self::new_unchecked(data) })
    }

    /// Creates a `SlotHashes` instance without validation.
    ///
    /// This is an unsafe constructor that bypasses all validation checks for performance.
    /// In debug builds, it still runs `parse_and_validate_data` as a sanity check.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not validate the data size or format.
    /// The caller must ensure:
    /// 1. The underlying byte slice in `data` represents valid SlotHashes data
    ///    (length prefix + entries, where entries are sorted in descending order by slot).
    /// 2. The data slice has at least `NUM_ENTRIES_SIZE + (declared_entries * ENTRY_SIZE)` bytes.
    /// 3. The first 8 bytes contain a valid entry count in little-endian format.
    ///
    #[inline(always)]
    pub unsafe fn new_unchecked(data: T) -> Self {
        if cfg!(debug_assertions) {
            parse_and_validate_data(&data)
                .expect("`data` matches all the same requirements as for `new()`");
        }

        SlotHashes { data }
    }

    /// Returns the number of `SlotHashEntry` items accessible.
    #[inline(always)]
    pub fn len(&self) -> usize {
        // SAFETY: `SlotHashes::new` and `new_unchecked` guarantee that `self.data` has at
        // least `NUM_ENTRIES_SIZE` bytes, so reading the entry count without additional
        // checks is safe.
        unsafe { read_entry_count_from_bytes_unchecked(&self.data) }
    }

    /// Returns if the sysvar is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a `&[SlotHashEntry]` view into the underlying data.
    ///
    /// Call once and reuse the slice if you need many look-ups.
    ///
    /// The constructor (in the safe path that called `parse_and_validate_data`)
    /// or caller (if unsafe `new_unchecked` path) is responsible for ensuring
    /// the slice is big enough and properly aligned.
    #[inline(always)]
    pub fn entries(&self) -> &[SlotHashEntry] {
        unsafe {
            // SAFETY: The slice begins `NUM_ENTRIES_SIZE` bytes into `self.data`, which
            // is guaranteed by parse_and_validate_data() to have at least `len * ENTRY_SIZE`
            // additional bytes. The pointer is properly aligned for `SlotHashEntry` (which
            // a compile-time assertion ensures is alignment 1).
            from_raw_parts(
                self.data.as_ptr().add(NUM_ENTRIES_SIZE) as *const SlotHashEntry,
                self.len(),
            )
        }
    }

    /// Gets a reference to the entry at `index` or `None` if out of bounds.
    #[inline(always)]
    pub fn get_entry(&self, index: usize) -> Option<&SlotHashEntry> {
        if index >= self.len() {
            return None;
        }
        Some(unsafe { self.get_entry_unchecked(index) })
    }

    /// Finds the hash for a specific slot using binary search.
    ///
    /// Returns the hash if the slot is found, or `None` if not found.
    /// Assumes entries are sorted by slot in descending order.
    /// If calling repeatedly, prefer getting `entries()` in caller
    /// to avoid repeated slice construction.
    #[inline(always)]
    pub fn get_hash(&self, target_slot: Slot) -> Option<&Hash> {
        self.position(target_slot)
            .map(|index| unsafe { &self.get_entry_unchecked(index).hash })
    }

    /// Finds the position (index) of a specific slot using binary search.
    ///
    /// Returns the index if the slot is found, or `None` if not found.
    /// Assumes entries are sorted by slot in descending order.
    /// If calling repeatedly, prefer getting `entries()` in caller
    /// to avoid repeated slice construction.
    #[inline(always)]
    pub fn position(&self, target_slot: Slot) -> Option<usize> {
        self.entries()
            .binary_search_by(|probe_entry| probe_entry.slot().cmp(&target_slot).reverse())
            .ok()
    }

    /// Returns a reference to the entry at `index` **without** bounds checking.
    ///
    /// # Safety
    /// Caller must guarantee that `index < self.len()`.
    #[inline(always)]
    pub unsafe fn get_entry_unchecked(&self, index: usize) -> &SlotHashEntry {
        debug_assert!(index < self.len());
        // SAFETY: Caller guarantees `index < self.len()`. The data pointer is valid
        // and aligned for `SlotHashEntry`. The offset calculation points to a
        // valid entry within the allocated data.
        let entries_ptr = self.data.as_ptr().add(NUM_ENTRIES_SIZE) as *const SlotHashEntry;
        &*entries_ptr.add(index)
    }
}

impl<'a, T: Deref<Target = [u8]>> IntoIterator for &'a SlotHashes<T> {
    type Item = &'a SlotHashEntry;
    type IntoIter = core::slice::Iter<'a, SlotHashEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries().iter()
    }
}

impl<'a> SlotHashes<Ref<'a, [u8]>> {
    /// Creates a `SlotHashes` instance by safely borrowing data from an `AccountInfo`.
    ///
    /// This function verifies that:
    /// - The account key matches the `SLOTHASHES_ID`
    /// - The account data can be successfully borrowed
    ///
    /// Returns a `SlotHashes` instance that borrows the account's data for zero-copy access.
    /// The returned instance is valid for the lifetime of the borrow.
    ///
    /// # Errors
    /// - `ProgramError::InvalidArgument` if the account key doesn't match the SlotHashes sysvar ID
    /// - `ProgramError::AccountBorrowFailed` if the account data is already mutably borrowed
    #[inline(always)]
    pub fn from_account_info(account_info: &'a AccountInfo) -> Result<Self, ProgramError> {
        if account_info.key() != &SLOTHASHES_ID {
            return Err(ProgramError::InvalidArgument);
        }

        let data_ref = account_info.try_borrow_data()?;

        // SAFETY: The account was validated to be the `SlotHashes` sysvar.
        Ok(unsafe { SlotHashes::new_unchecked(data_ref) })
    }
}

#[cfg(feature = "std")]
impl SlotHashes<Box<[u8]>> {
    /// Fills the provided buffer with the full SlotHashes sysvar data.
    ///
    /// # Safety
    /// The caller must ensure the buffer pointer is valid for MAX_SIZE bytes.
    /// The syscall will write exactly MAX_SIZE bytes to the buffer.
    #[inline(always)]
    unsafe fn fill_from_sysvar(buffer_ptr: *mut u8) -> Result<(), ProgramError> {
        crate::sysvars::get_sysvar_unchecked(buffer_ptr, &SLOTHASHES_ID, 0, MAX_SIZE)?;

        // For tests on builds that don't actually fill the buffer.
        #[cfg(not(target_os = "solana"))]
        core::ptr::write_bytes(buffer_ptr, 0, NUM_ENTRIES_SIZE);

        Ok(())
    }

    /// Allocates an optimal buffer for the sysvar data based on available features.
    #[inline(always)]
    fn allocate_and_fetch() -> Result<Box<[u8]>, ProgramError> {
        let mut buf = std::vec::Vec::with_capacity(MAX_SIZE);
        unsafe {
            // SAFETY: `buf` was allocated with capacity `MAX_SIZE` so its
            // pointer is valid for exactly that many bytes. `fill_from_sysvar`
            // writes `MAX_SIZE` bytes, and we immediately set the length to
            // `MAX_SIZE`, marking the entire buffer as initialized before it is
            // turned into a boxed slice.
            Self::fill_from_sysvar(buf.as_mut_ptr())?;
            buf.set_len(MAX_SIZE);
        }
        Ok(buf.into_boxed_slice())
    }

    /// Fetches the SlotHashes sysvar data directly via syscall. This copies
    /// the full sysvar data (`MAX_SIZE` bytes).
    #[inline(always)]
    pub fn fetch() -> Result<Self, ProgramError> {
        let data_init = Self::allocate_and_fetch()?;

        // SAFETY: The data was initialized by the syscall.
        Ok(unsafe { SlotHashes::new_unchecked(data_init) })
    }
}
