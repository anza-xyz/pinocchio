//! Raw / caller-supplied buffer helpers for the `SlotHashes` sysvar.
//!
//! This sub-module exposes lightweight functions that let a program copy
//! `SlotHashes` data directly into an arbitrary buffer **without** constructing
//! a `SlotHashes<T>` view. Use these when you only need a byte snapshot or
//! when including the sysvar account is infeasible.
#![allow(clippy::inline_always)]

use super::*;

/// Validates buffer format for `SlotHashes` data and calculates entry capacity.
///
/// Validates that the buffer follows the correct format:
/// - If `offset == 0`: Buffer must have `8 + (N × 40)` format (header and entries)
/// - If `offset != 0`: Buffer must be a multiple of 40 bytes (entries only)
///
/// Does not validate that `offset + buffer_len ≤ MAX_SIZE`; this is checked
/// separately in `validate_fetch_offset`, and the syscall will fail anyway if
/// `offset + buffer_len > MAX_SIZE`.
///
/// Returns the number of entries that can fit in the buffer.
#[inline(always)]
pub(crate) fn validate_and_get_buffer_capacity(
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
/// Returns the number of entries:
/// - If `offset == 0`: The actual entry count read from the sysvar header
/// - If `offset != 0`: The number of entries that can fit in the buffer
///
/// The return value helps callers understand the structure of the copied data.
#[inline(always)]
pub fn fetch_into(buffer: &mut [u8], offset: usize) -> Result<usize, ProgramError> {
    let num_entries = validate_and_get_buffer_capacity(buffer.len(), offset)?;

    validate_fetch_offset(offset, buffer.len())?;

    // SAFETY: Buffer format and offset alignment validated above.
    unsafe { fetch_into_unchecked(buffer, offset) }?;

    if offset == 0 {
        // Buffer includes header: return actual entry count from sysvar data
        Ok(read_entry_count_from_bytes(buffer).unwrap_or(0))
    } else {
        // Buffer excludes header: return calculated entry capacity
        Ok(num_entries)
    }
}

/// Copies `SlotHashes` sysvar bytes into `buffer` **without** validation.
///
/// The caller is responsible for ensuring that:
/// 1. `buffer` is large enough for the requested `offset + buffer.len()` range and
///    properly laid out (see `validate_buffer_size` and `validate_fetch_offset`).
/// 2. `offset + buffer.len()` is not greater than `MAX_SIZE`, or the syscall will
///    fail.
/// 3. The memory behind `buffer` is writable for its full length.
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
