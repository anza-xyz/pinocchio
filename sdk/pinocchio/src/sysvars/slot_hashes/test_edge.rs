use crate::{error::ProgramError, sysvars::slot_hashes::*};
extern crate std;
use super::test_utils::{build_slot_hashes_bytes as raw_slot_hashes, make_account_info};

#[test]
fn test_wrong_key_from_account_info() {
    let bytes = raw_slot_hashes(0, &[]);
    let (info, _backing) = unsafe {
        make_account_info(
            Address::new_from_array([1u8; 32]),
            &bytes,
            crate::entrypoint::NON_DUP_MARKER,
        )
    };
    assert!(matches!(
        SlotHashes::from_account_info(&info),
        Err(ProgramError::InvalidArgument)
    ));
}

#[test]
fn test_wrong_size_buffer_rejected() {
    // Buffer that declares 1 entry but is 1 byte too small to hold it.
    let num_entries: u64 = 1;
    let required_size = NUM_ENTRIES_SIZE + (num_entries as usize) * ENTRY_SIZE;
    let mut small_buffer = std::vec![0u8; required_size - 1];
    small_buffer[0..NUM_ENTRIES_SIZE].copy_from_slice(&num_entries.to_le_bytes());

    assert!(matches!(
        SlotHashes::new(small_buffer.as_slice()),
        Err(ProgramError::AccountDataTooSmall)
    ));

    // Buffer too small to even contain the length header.
    let too_small_for_header = [0u8; NUM_ENTRIES_SIZE - 1];
    assert!(matches!(
        SlotHashes::new(too_small_for_header.as_slice()),
        Err(ProgramError::AccountDataTooSmall)
    ));
}

#[test]
fn test_truncated_payload_with_max_size_buffer_is_valid() {
    let entry = (123u64, [7u8; HASH_BYTES]);
    let bytes = raw_slot_hashes(2, &[entry]); // says 2 but provides 1, rest is zeros

    // With MAX_SIZE buffers, this is now valid - the second entry is just zeros
    let slot_hashes = SlotHashes::new(bytes.as_slice()).expect("Should be valid");
    assert_eq!(slot_hashes.len(), 2);

    // First entry should match what we provided
    let first_entry = slot_hashes.get_entry(0).unwrap();
    assert_eq!(first_entry.slot(), 123);
    assert_eq!(first_entry.hash, [7u8; HASH_BYTES]);

    // Second entry should be all zeros (default padding)
    let second_entry = slot_hashes.get_entry(1).unwrap();
    assert_eq!(second_entry.slot(), 0);
    assert_eq!(second_entry.hash, [0u8; HASH_BYTES]);
}

#[test]
fn test_duplicate_slots_binary_search_safe() {
    let entries = &[
        (200, [0u8; HASH_BYTES]),
        (200, [1u8; HASH_BYTES]),
        (199, [2u8; HASH_BYTES]),
    ];
    let bytes = raw_slot_hashes(entries.len() as u64, entries);
    let sh = unsafe { SlotHashes::new_unchecked(&bytes[..]) };
    let dup_pos = sh.position(200).expect("slot 200 must exist");
    assert!(
        dup_pos <= 1,
        "binary_search should return one of the duplicate indices (0 or 1)"
    );
    assert_eq!(sh.get_hash(199), Some(&entries[2].1));
}

#[test]
fn test_zero_len_minimal_slice_iterates_empty() {
    let zero_data = raw_slot_hashes(0, &[]);
    let sh = unsafe { SlotHashes::new_unchecked(&zero_data[..]) };
    assert_eq!(sh.len(), 0);
    assert!(sh.into_iter().next().is_none());
}

#[test]
fn test_borrow_state_failure_from_account_info() {
    let bytes = raw_slot_hashes(0, &[]);
    let (info, _backing) = unsafe { make_account_info(SLOTHASHES_ID, &bytes, 0) };
    assert!(matches!(
        SlotHashes::from_account_info(&info),
        Err(ProgramError::AccountBorrowFailed)
    ));
}
