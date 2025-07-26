//! Raw / caller-supplied buffer helpers for the `SlotHashes` sysvar.
//!
//! This sub-module exposes lightweight functions that let a program copy
//! `SlotHashes` data directly into an arbitrary buffer **without** constructing
//! a `SlotHashes<T>` view. Use these when you only need a byte snapshot or
//! when including the sysvar account is infeasible.
#![allow(clippy::inline_always)]

use super::*;

/// Validates that a buffer is properly sized for `SlotHashes` data.
///
/// Does not ensure the buffer doesn't exceed available sysvar data
/// from the given offset. A later syscall will fail in this case.
///
/// This function assumes the mainnet slot hashes sysvar length of `MAX_SIZE` (20,488).
///
/// Returns the number of entries that will be copied into the buffer.
#[inline(always)]
pub(crate) fn validate_buffer_size(
    buffer_len: usize,
    offset: usize,
) -> Result<usize, ProgramError> {
    if offset == 0 {
        // Buffer includes header: must have 8 + (N × 40) format
        if buffer_len == MAX_SIZE {
            return Ok(MAX_ENTRIES);
        }

        if buffer_len < NUM_ENTRIES_SIZE {
            return Err(ProgramError::AccountDataTooSmall);
        }

        let entry_data_len = buffer_len - NUM_ENTRIES_SIZE;
        if entry_data_len % ENTRY_SIZE != 0 {
            return Err(ProgramError::InvalidArgument);
        }

        Ok(entry_data_len / ENTRY_SIZE)
    } else {
        // Buffer contains only entry data: must be multiple of ENTRY_SIZE
        if buffer_len % ENTRY_SIZE != 0 {
            return Err(ProgramError::InvalidArgument);
        }

        Ok(buffer_len / ENTRY_SIZE)
    }
}

/// Validates offset parameters for fetching `SlotHashes` data.
///
/// * `offset` - Byte offset within the `SlotHashes` sysvar data.
/// * `buffer_len` - Length of the destination buffer.
#[inline(always)]
pub fn validate_fetch_offset(offset: usize, buffer_len: usize) -> Result<(), ProgramError> {
    if offset >= MAX_SIZE {
        return Err(ProgramError::InvalidArgument);
    }
    if offset != 0 && (offset < NUM_ENTRIES_SIZE || (offset - NUM_ENTRIES_SIZE) % ENTRY_SIZE != 0) {
        return Err(ProgramError::InvalidArgument);
    }
    // Perhaps redundant, as the syscall will fail later if
    // `buffer.len() + offset > MAX_SIZE`, but this is for
    // checked paths.
    if offset.saturating_add(buffer_len) > MAX_SIZE {
        return Err(ProgramError::InvalidArgument);
    }

    Ok(())
}

/// Copies `SlotHashes` sysvar bytes into `buffer`, performing validation.
///
/// # Arguments
///
/// * `buffer` - Destination buffer to copy sysvar data into
/// * `offset` - Byte offset within the `SlotHashes` sysvar data to start copying from
///
/// # Returns
///
/// Since `num_entries` is used, it is returned for caller's convenience, as the
/// caller will almost certainly want to use this information.
#[inline(always)]
pub fn fetch_into(buffer: &mut [u8], offset: usize) -> Result<usize, ProgramError> {
    let num_entries = validate_buffer_size(buffer.len(), offset)?;

    validate_fetch_offset(offset, buffer.len())?;

    // SAFETY: `buffer.len()` and `offset` are both validated above. It is possible
    // that the two added together are greater than `MAX_SIZE`, but the syscall will
    // fail in that case.
    unsafe { fetch_into_unchecked(buffer, offset) }?;

    if offset == 0 {
        // If header is preset, read entries from it
        Ok(read_entry_count_from_bytes(buffer).unwrap_or(0))
    } else {
        // Otherwise, return the number of entries that can fit in the buffer
        Ok(num_entries)
    }
}

/// Copies `SlotHashes` sysvar bytes into `buffer` **without** validation.
///
/// The caller is responsible for ensuring that:
/// 1. `buffer` is large enough for the requested `offset + buffer.len()` range and
///    properly laid out (see `validate_buffer_size` and `validate_fetch_offset`).
/// 2. The memory behind `buffer` is writable for its full length.
///
/// # Safety
/// Internally this function performs an unchecked Solana syscall that writes
/// raw bytes into the provided pointer.
#[inline(always)]
pub unsafe fn fetch_into_unchecked(buffer: &mut [u8], offset: usize) -> Result<(), ProgramError> {
    crate::sysvars::get_sysvar_unchecked(
        buffer.as_mut_ptr(),
        &SLOTHASHES_ID,
        offset,
        buffer.len(),
    )?;

    Ok(())
}
