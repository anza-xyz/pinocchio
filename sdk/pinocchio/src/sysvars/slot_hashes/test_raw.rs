//! Tests focusing on low-level `slot_hashes::raw` helpers.

use super::raw;
use super::*;
extern crate std;

#[test]
fn test_validate_buffer_size() {
    let small_len = 4;
    assert!(raw::validate_buffer_size(small_len).is_err());

    let misaligned_len = NUM_ENTRIES_SIZE + 39;
    assert!(raw::validate_buffer_size(misaligned_len).is_err());

    let oversized_len = NUM_ENTRIES_SIZE + (MAX_ENTRIES + 1) * ENTRY_SIZE;
    assert!(raw::validate_buffer_size(oversized_len).is_err());

    let valid_empty_len = NUM_ENTRIES_SIZE;
    assert!(raw::validate_buffer_size(valid_empty_len).is_ok());

    let valid_one_len = NUM_ENTRIES_SIZE + ENTRY_SIZE;
    assert!(raw::validate_buffer_size(valid_one_len).is_ok());

    let valid_max_len = NUM_ENTRIES_SIZE + MAX_ENTRIES * ENTRY_SIZE;
    assert!(raw::validate_buffer_size(valid_max_len).is_ok());

    // Edge case: exactly at the boundary
    let boundary_len = NUM_ENTRIES_SIZE + MAX_ENTRIES * ENTRY_SIZE;
    assert!(raw::validate_buffer_size(boundary_len).is_ok());
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
    let mut full = std::vec![0u8; MAX_SIZE];
    let n = raw::fetch_into(&mut full, 0).expect("fetch_into(full, 0)");
    assert_eq!(n, 0);

    // 2. Header-only buffer.
    let mut header_only = std::vec![0u8; NUM_ENTRIES_SIZE];
    let n2 = raw::fetch_into(&mut header_only, 0).expect("fetch_into(header_only, 0)");
    assert_eq!(n2, 0);

    // 3. One-entry buffer.
    let mut one_entry = std::vec![0u8; NUM_ENTRIES_SIZE + ENTRY_SIZE];
    let n3 = raw::fetch_into(&mut one_entry, 0).expect("fetch_into(one_entry, 0)");
    assert_eq!(n3, 0);

    // 4. Header-skipped fetch should fail because header is missing.
    let mut skip_header = std::vec![0u8; ENTRY_SIZE];
    assert!(raw::fetch_into(&mut skip_header, 8).is_err());

    // 5. Mis-aligned buffer size should error.
    let mut misaligned = std::vec![0u8; NUM_ENTRIES_SIZE + 39];
    assert!(raw::fetch_into(&mut misaligned, 0).is_err());

    // 6. Mid-entry offset should error.
    let mut buf = std::vec![0u8; 64];
    assert!(raw::fetch_into(&mut buf, 12).is_err());

    // 7. Offset + len overflow should error.
    let mut small = std::vec![0u8; 200];
    assert!(raw::fetch_into(&mut small, MAX_SIZE - 199).is_err());
}
