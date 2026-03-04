//! Tests focusing on low-level `slot_hashes::raw` helpers.

#![allow(clippy::undocumented_unsafe_blocks, clippy::multiple_unsafe_ops_per_block)]

use super::{raw, *};

#[test]
fn test_validate_buffer_size() {
    // ===== Tests with offset = 0 (buffer includes header) =====

    // Too small to fit header
    let small_len = 4;
    assert!(raw::get_valid_buffer_capacity(small_len, 0).is_err());

    // Misaligned: header + partial entry
    let misaligned_len = NUM_ENTRIES_SIZE + 39;
    assert!(raw::get_valid_buffer_capacity(misaligned_len, 0).is_err());

    // Valid cases with offset = 0
    let valid_empty_len = NUM_ENTRIES_SIZE;
    assert_eq!(
        raw::get_valid_buffer_capacity(valid_empty_len, 0).unwrap(),
        0
    );

    let valid_one_len = NUM_ENTRIES_SIZE + ENTRY_SIZE;
    assert_eq!(raw::get_valid_buffer_capacity(valid_one_len, 0).unwrap(), 1);

    let valid_max_len = NUM_ENTRIES_SIZE + MAX_ENTRIES * ENTRY_SIZE;
    assert_eq!(
        raw::get_valid_buffer_capacity(valid_max_len, 0).unwrap(),
        MAX_ENTRIES
    );

    // Edge case: exactly at the boundary (MAX_SIZE)
    assert_eq!(
        raw::get_valid_buffer_capacity(MAX_SIZE, 0).unwrap(),
        MAX_ENTRIES
    );

    // ===== Tests with offset != 0 (buffer doesn't include header) =====

    // Valid cases with non-zero offset - buffer contains only entry data

    // Buffer for exactly 1 entry
    assert_eq!(raw::get_valid_buffer_capacity(ENTRY_SIZE, 8).unwrap(), 1);

    // Buffer for exactly 2 entries
    assert_eq!(
        raw::get_valid_buffer_capacity(2 * ENTRY_SIZE, 8).unwrap(),
        2
    );

    // Buffer for maximum entries (without header space)
    assert_eq!(
        raw::get_valid_buffer_capacity(MAX_ENTRIES * ENTRY_SIZE, 8).unwrap(),
        MAX_ENTRIES
    );

    // Buffer for 10 entries
    assert_eq!(
        raw::get_valid_buffer_capacity(10 * ENTRY_SIZE, 48).unwrap(),
        10
    );

    // Error cases with non-zero offset

    // Misaligned buffer - not a multiple of ENTRY_SIZE
    assert!(raw::get_valid_buffer_capacity(ENTRY_SIZE + 1, 8).is_err());
    assert!(raw::get_valid_buffer_capacity(ENTRY_SIZE - 1, 8).is_err());
    assert!(raw::get_valid_buffer_capacity(39, 8).is_err()); // 39 is not divisible by 40

    // Large buffers that would exceed MAX_SIZE - these now pass
    // validate_buffer_size (the syscall will fail later, but that's acceptable)
    assert_eq!(
        raw::get_valid_buffer_capacity((MAX_ENTRIES + 1) * ENTRY_SIZE, 8).unwrap(),
        MAX_ENTRIES + 1
    );
    assert_eq!(
        raw::get_valid_buffer_capacity((MAX_ENTRIES + 10) * ENTRY_SIZE, 48).unwrap(),
        MAX_ENTRIES + 10
    );

    // Empty buffer with offset (valid - 0 entries)
    assert_eq!(raw::get_valid_buffer_capacity(0, 8).unwrap(), 0);

    // ===== Additional edge cases =====

    // Large offset values (should still work for buffer size validation)
    assert_eq!(
        raw::get_valid_buffer_capacity(5 * ENTRY_SIZE, 1000).unwrap(),
        5
    );
    assert!(raw::get_valid_buffer_capacity(5 * ENTRY_SIZE + 1, 2000).is_err());
    // misaligned
}

#[test]
fn test_fetch_into_offset_validation() {
    let buffer_len = 200;

    // Offset 0 (start of data) - should pass validation
    assert!(validate_fetch_offset(0, buffer_len).is_ok());

    // Offset 8 (start of first entry) - should pass validation
    assert!(validate_fetch_offset(8, buffer_len).is_ok());

    // Offset 48 (start of second entry) - should pass validation
    assert!(validate_fetch_offset(48, buffer_len).is_ok());

    // Offset 88 (start of third entry) - should pass validation
    assert!(validate_fetch_offset(88, buffer_len).is_ok());

    // Invalid offsets that should fail validation

    // Offset beyond MAX_SIZE
    assert!(validate_fetch_offset(MAX_SIZE, buffer_len).is_err());

    // Offset pointing mid-entry (not aligned)
    assert!(validate_fetch_offset(12, buffer_len).is_err()); // 8 + 4, mid-entry
    assert!(validate_fetch_offset(20, buffer_len).is_err()); // 8 + 12, mid-entry
    assert!(validate_fetch_offset(35, buffer_len).is_err()); // 8 + 27, mid-entry

    // Offset in header but not at start
    assert!(validate_fetch_offset(4, buffer_len).is_err()); // Mid-header
    assert!(validate_fetch_offset(7, buffer_len).is_err()); // End of header

    // Test buffer + offset exceeding MAX_SIZE
    assert!(validate_fetch_offset(1, MAX_SIZE).is_err());
    assert!(validate_fetch_offset(MAX_SIZE - 100, 200).is_err());

    // Last entry
    assert!(validate_fetch_offset(8 + 511 * ENTRY_SIZE, 40).is_ok());

    // One past last valid entry
    assert!(validate_fetch_offset(8 + 512 * ENTRY_SIZE, 40).is_err());
}

/// Host-only smoke test for `raw::fetch_into`.
///
/// On a host build the underlying sysvar syscall is stubbed out.
#[test]
fn test_fetch_into_host_stub() {
    // 1. Full-size buffer, offset 0.
    let mut full = alloc::vec![0u8; MAX_SIZE];
    let n = raw::fetch_into(&mut full, 0).expect("fetch_into(full, 0)");
    assert_eq!(n, 0);

    // 2. Header-only buffer.
    let mut header_only = alloc::vec![0u8; NUM_ENTRIES_SIZE];
    let n2 = raw::fetch_into(&mut header_only, 0).expect("fetch_into(header_only, 0)");
    assert_eq!(n2, 0);

    // 3. One-entry buffer.
    let mut one_entry = alloc::vec![0u8; NUM_ENTRIES_SIZE + ENTRY_SIZE];
    let n3 = raw::fetch_into(&mut one_entry, 0).expect("fetch_into(one_entry, 0)");
    assert_eq!(n3, 0);

    // 4. Header-skipped fetch should succeed and return the number of entries that
    //    fit.
    let mut skip_header = alloc::vec![0u8; ENTRY_SIZE];
    let entries_count = raw::fetch_into(&mut skip_header, 8).expect("fetch_into(skip_header, 8)");
    assert_eq!(entries_count, 1); // Buffer can fit exactly 1 entry

    // 5. Mis-aligned buffer size should error.
    let mut misaligned = alloc::vec![0u8; NUM_ENTRIES_SIZE + 39];
    assert!(raw::fetch_into(&mut misaligned, 0).is_err());

    // 6. Mid-entry offset should error.
    let mut buf = alloc::vec![0u8; 64];
    assert!(raw::fetch_into(&mut buf, 12).is_err());

    // 7. Offset + len overflow should error.
    let mut small = alloc::vec![0u8; 200];
    assert!(raw::fetch_into(&mut small, MAX_SIZE - 199).is_err());
}

/// Test that `fetch_into` with offset correctly avoids interpreting slot
/// data as entry count.
#[test]
fn test_fetch_into_offset_avoids_incorrect_entry_count() {
    // When fetch_into is called with offset != 0, the first
    // 8 bytes of the buffer contains header data, not entry data.
    let mut buffer = alloc::vec![0u8; 3 * ENTRY_SIZE];

    // Call fetch_into with offset 8 (skipping the 8-byte header)
    let result = raw::fetch_into(&mut buffer, 8);

    assert!(
        result.is_ok(),
        "fetch_into should succeed with offset that skips header"
    );

    let entries_that_fit = result.unwrap();
    assert_eq!(
        entries_that_fit, 3,
        "Should return number of entries that fit in buffer, not some slot number"
    );

    // Buffer for exactly 1 entry starting from offset 48 (2nd entry)
    let mut second_entry_buffer = alloc::vec![0u8; ENTRY_SIZE];
    let second_result = raw::fetch_into(&mut second_entry_buffer, 48).unwrap();
    assert_eq!(second_result, 1);
}
