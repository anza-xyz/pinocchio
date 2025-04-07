//! SlotHashes sysvar implementation

use super::Sysvar;
use crate::{
    program_error::ProgramError, 
    pubkey::Pubkey, 
    sysvars::clock::Slot
};

/// The ID of the slot hashes sysvar.
pub const SLOT_HASHES_ID: Pubkey = [ 6, 167, 213, 23, 25, 47, 10, 175, 200, 117, 226, 225, 132, 
    87, 124, 80, 105, 207, 200, 70, 73, 227, 235, 146, 120, 47, 149, 141, 72, 0, 0, 0 ];

/// Hash type used to track slot hashes
pub type Hash = [u8; 32];

/// Maximum number of entries in the slot hashes sysvar.
pub const MAX_ENTRIES: usize = 512;

/// Size of a slot value in bytes.
pub const SLOT_SIZE: usize = core::mem::size_of::<Slot>();

/// Size of a hash value in bytes.
pub const HASH_SIZE: usize = core::mem::size_of::<Hash>();

/// Size of a slot-hash pair in bytes.
pub const PAIR_SIZE: usize = SLOT_SIZE + HASH_SIZE;

/// Size of the vector length field in bytes.
pub const VEC_LENGTH_SIZE: usize = core::mem::size_of::<u64>();

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotHashEntry {
    /// slot number
    pub slot: Slot,

    /// hash value
    pub hash: Hash
}

/// A struct to access the slot hashes sysvar.
///
/// This struct provides efficient methods to access specific entries in the
/// slot hashes sysvar without deserializing the entire data structure, which
/// would be too large to process on-chain.
#[derive(Debug, Default)]
pub struct SlotHashesSysvar;

impl SlotHashesSysvar {
    /// Returns the number of entries in the slot hashes sysvar.
    ///
    /// Returns `None` if the sysvar syscall fails.
    pub fn len() -> Option<usize> {
      let mut len_buf = [0u8; VEC_LENGTH_SIZE];

      let result = unsafe {
          crate::syscalls::sol_get_sysvar(
            SLOT_HASHES_ID.as_ptr(),
            len_buf.as_mut_ptr(),
            0,
            VEC_LENGTH_SIZE as u64,
          )
      };

      if result != 0 {
        return None;
      }

      let len = u64::from_le_bytes(len_buf);
      Some(core::cmp::min(len as usize, MAX_ENTRIES))
    }

    /// Checks if the slot hashes sysvar is empty.
    ///
    /// Returns `None` if the sysvar syscall fails.
    pub fn is_empty()-> Option<bool> {
        Self::len().map(|len| len == 0)
    }

    /// Gets the slot hash entry at the specified index.
    ///
    /// Returns `None` if the index is out of bounds or the sysvar syscall fails.
    pub fn get_entry(index: usize) -> Option<SlotHashEntry> {
        let len = Self::len()?;

        if index >= len {
            return None;
        }

        let offset = (VEC_LENGTH_SIZE + (index * PAIR_SIZE)) as u64;
        let mut entry_buf = [0u8, PAIR_SIZE as u8];

        let result = unsafe {
            crate::syscalls::sol_get_sysvar(
                SLOT_HASHES_ID.as_ptr(), 
                entry_buf.as_mut_ptr(), 
                offset, 
                PAIR_SIZE as u64
            )
        };

        if result != 0 {
            return None;
        }

        // First 8 bytes are the Slot
        let slot = u64::from_le_bytes([
            entry_buf[0], entry_buf[1], entry_buf[2], entry_buf[3],
            entry_buf[4], entry_buf[5], entry_buf[6], entry_buf[7],
        ]);

        // Next 32 bytes are the hash
        let mut hash = [0u8; HASH_SIZE];
        hash.copy_from_slice(&entry_buf[SLOT_SIZE..]);

        Some(SlotHashEntry { slot, hash })
    }

    /// Gets the hash for the specified slot.
    ///
    /// This performs a linear search through the entries.
    /// Returns `None` if the slot is not found or the sysvar syscall fails.
    pub fn get_hash(target_slot: Slot) -> Option<Hash> {
        let len = Self::len()?;

        for i in 0..len {
            if let Some(entry) = Self::get_entry(i) {
                if entry.slot == target_slot {
                    return Some(entry.hash);
                } else {
                    break;
                }
            }
        }

        None
    }

    /// Gets the hash for the specified slot.
    ///
    /// This performs a linear search through the entries.
    /// Returns `None` if the slot is not found or the sysvar syscall fails.
    pub fn position(target_slot: Slot) -> Option<usize> {
        let len = Self::len()?;

        for i in 0..len {
            if let Some(entry) = Self::get_entry(i) {
                if entry.slot == target_slot {
                    return Some(i);
                } else {
                    break;
                }
            }
        }

        None
    }
}

/// The slot hashes sysvar is too large to deserialize on-chain using the standard
/// `Sysvar::get()` method. Use the static methods provided by `SlotHashesSysvar`
/// (like `get_entry()`, `get_hash()`, and `len()`) to access specific entries instead.
impl Sysvar for SlotHashesSysvar {
    fn get() -> Result<Self, ProgramError> {
        Err(ProgramError::UnsupportedSysvar)
    }
}






