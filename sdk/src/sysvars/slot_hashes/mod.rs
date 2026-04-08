//! Efficient, zero-copy access to `SlotHashes` sysvar data.

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

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use {
    crate::{
        account::{AccountView, Ref},
        error::ProgramError,
        hint::unlikely,
        sysvars::clock::Slot,
        Address,
    },
    core::{mem, ops::Deref, slice::from_raw_parts},
};

/// `SysvarS1otHashes111111111111111111111111111`
pub const SLOTHASHES_ID: Address = Address::new_from_array([
    6, 167, 213, 23, 25, 47, 10, 175, 198, 242, 101, 227, 251, 119, 204, 122, 218, 130, 197, 41,
    208, 190, 59, 19, 110, 45, 0, 85, 32, 0, 0, 0,
]);
/// Number of bytes in a hash.
pub const HASH_BYTES: usize = 32;
/// Sysvar data is:
/// `len`    (8 bytes): little-endian entry count (`≤ 512`)
/// `entries`(`len × 40 bytes`):    consecutive `(u64 slot, [u8;32] hash)` pairs
/// Size of the entry count field at the beginning of sysvar data.
pub const NUM_ENTRIES_SIZE: usize = mem::size_of::<u64>();
/// Size of a slot number in bytes.
pub const SLOT_SIZE: usize = mem::size_of::<Slot>();
/// Size of a single slot hash entry.
pub const ENTRY_SIZE: usize = SLOT_SIZE + HASH_BYTES;
/// Maximum number of slot hash entries that can be stored in the sysvar.
pub const MAX_ENTRIES: usize = 512;
/// Max size of the sysvar data in bytes. 20488. Golden on mainnet (never
/// smaller)
pub const MAX_SIZE: usize = NUM_ENTRIES_SIZE + MAX_ENTRIES * ENTRY_SIZE;
/// A single hash.
pub type Hash = [u8; HASH_BYTES];

/// A single entry in the `SlotHashes` sysvar.
#[cfg_attr(feature = "copy", derive(Copy))]
#[derive(Clone, Eq, Debug, PartialEq)]
#[repr(C)]
pub struct SlotHashEntry {
    /// The slot number stored as little-endian bytes.
    slot_le: [u8; 8],
    /// The hash corresponding to the slot.
    pub hash: Hash,
}

// Fail compilation if `SlotHashEntry` is not byte-aligned.
const _: [(); 1] = [(); mem::align_of::<SlotHashEntry>()];

/// `SlotHashes` provides read-only, zero-copy access to `SlotHashes` sysvar
/// bytes.
#[derive(Debug)]
pub struct SlotHashes<T: Deref<Target = [u8]>> {
    data: T,
}

/// Log a `Hash` from a program.
pub fn log(hash: &Hash) {
    #[cfg(any(target_os = "solana", target_arch = "bpf"))]
    // SAFETY: `sol_log_pubkey` expects a valid pointer to a 32-byte array.
    unsafe {
        solana_address::syscalls::sol_log_pubkey(hash.as_ptr());
    }

    #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
    core::hint::black_box(hash);
}

/// Get the number of entries from the sysvar data bytes.
///
/// # Safety
///
/// Caller must ensure that `data` has at least `NUM_ENTRIES_SIZE` bytes.
#[inline(always)]
pub(crate) unsafe fn get_entry_count(data: &[u8]) -> usize {
    debug_assert!(data.len() >= NUM_ENTRIES_SIZE);
    u64::from_le_bytes(*(data.as_ptr() as *const [u8; NUM_ENTRIES_SIZE])) as usize
}

impl SlotHashEntry {
    /// Returns the slot number as a `u64`.
    #[inline(always)]
    pub fn slot(&self) -> Slot {
        u64::from_le_bytes(self.slot_le)
    }
}

impl<T: Deref<Target = [u8]>> SlotHashes<T> {
    /// Creates a `SlotHashes` instance with validation of the entry count and
    /// buffer size.
    ///
    /// This constructor validates that the buffer has at least enough bytes to
    /// contain the declared number of entries. The buffer can be any size
    /// above the minimum required, making it suitable for both full
    /// `MAX_SIZE` buffers and smaller test data. Does not validate that
    /// entries are sorted in descending order.
    #[inline(always)]
    pub fn new(data: T) -> Result<Self, ProgramError> {
        if data.len() < NUM_ENTRIES_SIZE {
            return Err(ProgramError::AccountDataTooSmall);
        }

        // SAFETY: `data` is guaranteed to have at least `NUM_ENTRIES_SIZE` bytes.
        let num_entries = unsafe { get_entry_count(data.as_ref()) };

        if num_entries > MAX_ENTRIES {
            return Err(ProgramError::InvalidArgument);
        }

        // `num_entries` is guaranteed to be at most `MAX_ENTRIES`, so the
        // multiplication cannot overflow.
        let required_size = NUM_ENTRIES_SIZE + num_entries * ENTRY_SIZE;

        if data.len() < required_size {
            return Err(ProgramError::AccountDataTooSmall);
        }

        // SAFETY: `num_entries` is validated to be at most `MAX_ENTRIES`, and
        // `data.len()` is validated to be of the expected size.
        Ok(unsafe { Self::new_unchecked(data) })
    }

    /// Creates a `SlotHashes` instance without validation.
    ///
    /// This is an unsafe constructor that bypasses all validation checks for
    /// performance. In debug builds, it still runs
    /// `parse_and_validate_data` as a sanity check.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not validate the data size or
    /// format. The caller must ensure:
    /// 1. The underlying byte slice in `data` represents valid `SlotHashes`
    ///    data (length prefix plus entries, where entries are sorted in
    ///    descending order by slot).
    /// 2. The data slice has at least `NUM_ENTRIES_SIZE + (declared_entries *
    ///    ENTRY_SIZE)` bytes.
    /// 3. The first 8 bytes contain a valid entry count in little-endian
    ///    format.
    #[inline(always)]
    pub unsafe fn new_unchecked(data: T) -> Self {
        SlotHashes { data }
    }

    /// Returns the number of `SlotHashEntry` items accessible.
    #[inline(always)]
    pub fn len(&self) -> usize {
        // SAFETY: `SlotHashes` invariants guarantee that `self.data` has at least
        // `NUM_ENTRIES_SIZE` bytes.
        unsafe { get_entry_count(self.data.as_ref()) }
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
            // is guaranteed by parse_and_validate_data() to have at least `len *
            // ENTRY_SIZE` additional bytes. The pointer is properly aligned for
            // `SlotHashEntry` (which a compile-time assertion ensures is
            // alignment 1).
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
    /// Creates a `SlotHashes` instance by safely borrowing data from an
    /// `AccountView`.
    ///
    /// This function verifies that:
    /// - The account key matches the `SLOTHASHES_ID`
    /// - The account data can be successfully borrowed
    ///
    /// Returns a `SlotHashes` instance that borrows the account's data for
    /// zero-copy access. The returned instance is valid for the lifetime of
    /// the borrow.
    ///
    /// # Errors
    /// - `ProgramError::InvalidArgument` if the account key doesn't match the
    ///   `SlotHashes` sysvar ID
    /// - `ProgramError::AccountBorrowFailed` if the account data is already
    ///   mutably borrowed
    #[inline(always)]
    pub fn from_account_view(account_view: &'a AccountView) -> Result<Self, ProgramError> {
        if unlikely(account_view.address() != &SLOTHASHES_ID) {
            return Err(ProgramError::InvalidArgument);
        }

        let sysvar_data = account_view.try_borrow()?;

        // SAFETY: The account was validated to be the `SlotHashes` sysvar.
        Ok(unsafe { SlotHashes::new_unchecked(sysvar_data) })
    }
}

#[cfg(feature = "alloc")]
impl SlotHashes<Box<[u8]>> {
    /// Fetches the `SlotHashes` sysvar data directly via syscall.
    ///
    /// This copies the full sysvar data (`MAX_SIZE` bytes).
    #[inline(always)]
    pub fn fetch() -> Result<Self, ProgramError> {
        let mut sysvar_data = Box::<[u8]>::new_uninit_slice(MAX_SIZE);
        unsafe {
            #[cfg(any(target_os = "solana", target_arch = "bpf"))]
            crate::sysvars::get_sysvar_unchecked(
                sysvar_data.as_mut_ptr() as *mut _,
                &SLOTHASHES_ID,
                0,
                MAX_SIZE,
            )?;

            #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
            core::ptr::write_bytes(sysvar_data.as_mut_ptr(), 0, MAX_SIZE);
        }
        // SAFETY: The data was initialized by the syscall.
        Ok(unsafe { SlotHashes::new_unchecked(sysvar_data.assume_init()) })
    }
}
