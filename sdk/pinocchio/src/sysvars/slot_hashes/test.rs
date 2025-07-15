use crate::{
    program_error::ProgramError,
    sysvars::{clock::Slot, slot_hashes::*},
};
use core::mem::{align_of, size_of};
extern crate std;
use super::test_utils::*;
use std::vec::Vec;

#[test]
fn test_layout_constants() {
    assert_eq!(NUM_ENTRIES_SIZE, size_of::<u64>());
    assert_eq!(SLOT_SIZE, size_of::<u64>());
    assert_eq!(HASH_BYTES, 32);
    assert_eq!(ENTRY_SIZE, size_of::<u64>() + 32);
    assert_eq!(MAX_SIZE, 20_488);
    assert_eq!(size_of::<SlotHashEntry>(), ENTRY_SIZE);
    assert_eq!(align_of::<SlotHashEntry>(), align_of::<[u8; 8]>());
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

#[test]
fn test_binary_search_no_std() {
    const TEST_NUM_ENTRIES: usize = 512;
    const START_SLOT: u64 = 2000;

    let entries =
        generate_mock_entries(TEST_NUM_ENTRIES, START_SLOT, DecrementStrategy::Average1_05);
    let data = create_mock_data(&entries);
    let entry_count = entries.len();

    let first_slot = entries[0].0;
    let mid_index = entry_count / 2;
    let mid_slot = entries[mid_index].0;
    let last_slot = entries[entry_count - 1].0;

    let slot_hashes = unsafe { SlotHashes::new_unchecked(data.as_slice()) };

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

    assert_eq!(slot_hashes.position(START_SLOT + 1), None);

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
    let empty_hashes = unsafe { SlotHashes::new_unchecked(empty_data.as_slice()) };
    assert_eq!(empty_hashes.get_hash(100), None);

    let pos_start_plus_1 = slot_hashes.position(START_SLOT + 1);
    assert!(
        pos_start_plus_1.is_none(),
        "position(START_SLOT + 1) should be None"
    );
}

#[test]
fn test_basic_getters_and_iterator_no_std() {
    const NUM_ENTRIES: usize = 512;
    const START_SLOT: u64 = 2000;
    let entries = generate_mock_entries(NUM_ENTRIES, START_SLOT, DecrementStrategy::Strictly1);
    let data = create_mock_data(&entries);
    let slot_hashes = unsafe { SlotHashes::new_unchecked(data.as_slice()) };

    assert_eq!(slot_hashes.len(), NUM_ENTRIES);

    let entry0 = slot_hashes.get_entry(0);
    assert!(entry0.is_some());
    assert_eq!(entry0.unwrap().slot(), START_SLOT); // Check against start slot
    assert_eq!(entry0.unwrap().hash, [0u8; HASH_BYTES]); // First generated hash is [0u8; 32]

    let entry2 = slot_hashes.get_entry(NUM_ENTRIES - 1); // Last entry
    assert!(entry2.is_some());
    assert_eq!(entry2.unwrap().slot(), entries[NUM_ENTRIES - 1].0);
    assert_eq!(entry2.unwrap().hash, entries[NUM_ENTRIES - 1].1);
    assert!(slot_hashes.get_entry(NUM_ENTRIES).is_none()); // Out of bounds

    for (i, entry) in slot_hashes.into_iter().enumerate() {
        assert_eq!(entry.slot(), entries[i].0);
        assert_eq!(entry.hash, entries[i].1);
    }
    assert!(slot_hashes.into_iter().nth(NUM_ENTRIES).is_none());

    // Test ExactSizeIterator hint
    let mut iter_hint = slot_hashes.into_iter();
    assert_eq!(iter_hint.len(), NUM_ENTRIES);
    iter_hint.next();
    assert_eq!(iter_hint.len(), NUM_ENTRIES - 1);
    // Skip to end
    for _ in 1..NUM_ENTRIES {
        iter_hint.next();
    }
    iter_hint.next();
    assert_eq!(iter_hint.len(), 0);

    // Test empty case
    let empty_data = create_mock_data(&[]);
    let empty_hashes = unsafe { SlotHashes::new_unchecked(empty_data.as_slice()) };
    assert_eq!(empty_hashes.len(), 0);
    assert!(empty_hashes.get_entry(0).is_none());
    assert!(empty_hashes.into_iter().next().is_none());
}

#[test]
fn test_entry_count_no_std() {
    // Valid data (2 entries)
    let entries: &[(Slot, [u8; HASH_BYTES])] = &[(100, [1u8; HASH_BYTES]), (98, [2u8; HASH_BYTES])];
    let data = create_mock_data(entries);
    let slot_hashes = unsafe { SlotHashes::new_unchecked(data.as_slice()) };
    assert_eq!(slot_hashes.len(), 2);

    // Too small buffer should fail new()
    let num_entries = entries.len() as u64;
    let data_len = NUM_ENTRIES_SIZE + entries.len() * ENTRY_SIZE;
    let mut small_data = std::vec![0u8; data_len];
    small_data[0..NUM_ENTRIES_SIZE].copy_from_slice(&num_entries.to_le_bytes());
    let mut offset = NUM_ENTRIES_SIZE;
    for (slot, hash) in entries {
        small_data[offset..offset + SLOT_SIZE].copy_from_slice(&slot.to_le_bytes());
        small_data[offset + SLOT_SIZE..offset + ENTRY_SIZE].copy_from_slice(hash);
        offset += ENTRY_SIZE;
    }
    let res1 = SlotHashes::new(small_data.as_slice());
    assert!(
        res1.is_ok(),
        "SlotHashes::new should succeed with a correctly sized buffer"
    );
    let slot_hashes_from_small = res1.unwrap();
    assert_eq!(slot_hashes_from_small.len(), entries.len());

    // Empty data is valid
    let empty_data = create_mock_data(&[]);
    let empty_hashes = unsafe { SlotHashes::new_unchecked(empty_data.as_slice()) };
    assert_eq!(empty_hashes.len(), 0);
}

#[test]
fn test_get_entry_unchecked_no_std() {
    let single_entry: &[(Slot, [u8; HASH_BYTES])] = &[(100, [1u8; HASH_BYTES])];
    let data = create_mock_data(single_entry);
    let slot_hashes = unsafe { SlotHashes::new_unchecked(data.as_slice()) };

    let entry = unsafe { slot_hashes.get_entry_unchecked(0) };
    assert_eq!(entry.slot(), 100);
    assert_eq!(entry.hash, [1u8; HASH_BYTES]);
}

#[test]
fn test_get_entry_unchecked_last_no_std() {
    const COUNT: usize = 8;
    const START_SLOT: u64 = 600;
    let entries = generate_mock_entries(COUNT, START_SLOT, DecrementStrategy::Strictly1);
    let data = create_mock_data(&entries);
    let sh = unsafe { SlotHashes::new_unchecked(data.as_slice()) };

    let last = unsafe { sh.get_entry_unchecked(COUNT - 1) };
    assert_eq!(last.slot(), entries[COUNT - 1].0);
    assert_eq!(last.hash, entries[COUNT - 1].1);
}

#[test]
fn test_iterator_into_ref_no_std() {
    const NUM: usize = 16;
    const START: u64 = 100;
    let entries = generate_mock_entries(NUM, START, DecrementStrategy::Strictly1);
    let data = create_mock_data(&entries);
    let sh = unsafe { SlotHashes::new_unchecked(data.as_slice()) };

    // Collect slots via iterator
    let mut sum: u64 = 0;
    for e in &sh {
        sum += e.slot();
    }
    let expected_sum: u64 = entries.iter().map(|(s, _)| *s).sum();
    assert_eq!(sum, expected_sum);

    let iter = (&sh).into_iter();
    assert_eq!(iter.len(), sh.len());
}

// Tests to verify mock data helpers
#[test]
fn mock_data_max_entries_boundary() {
    let entries = generate_mock_entries(MAX_ENTRIES, 1000, DecrementStrategy::Strictly1);
    let data = create_mock_data(&entries);
    let sh = unsafe { SlotHashes::new_unchecked(data.as_slice()) };
    assert_eq!(sh.len(), MAX_ENTRIES);
}

#[test]
fn mock_data_raw_byte_layout() {
    let entries = &[(100u64, [0xAB; 32])];
    let data = create_mock_data(entries);
    // length prefix
    assert_eq!(&data[0..8], &1u64.to_le_bytes());
    // slot bytes
    assert_eq!(&data[8..16], &100u64.to_le_bytes());
    // hash bytes
    assert_eq!(&data[16..48], &[0xAB; 32]);
}

#[test]
fn test_read_entry_count_from_bytes() {
    let entry_count = 42u64;
    let mut data = [0u8; 16];
    data[0..8].copy_from_slice(&entry_count.to_le_bytes());

    let result = read_entry_count_from_bytes(&data);
    assert_eq!(result, Some(42));

    let zero_count = 0u64;
    let mut zero_data = [0u8; 8];
    zero_data.copy_from_slice(&zero_count.to_le_bytes());

    let zero_result = read_entry_count_from_bytes(&zero_data);
    assert_eq!(zero_result, Some(0));

    let max_count = MAX_ENTRIES as u64;
    let mut max_data = [0u8; 8];
    max_data.copy_from_slice(&max_count.to_le_bytes());

    let max_result = read_entry_count_from_bytes(&max_data);
    assert_eq!(max_result, Some(MAX_ENTRIES));
}

fn mock_fetch_into_unchecked(
    mock_sysvar_data: &[u8],
    buffer: &mut [u8],
    offset: u64,
) -> Result<(), ProgramError> {
    let offset = offset as usize;
    if offset >= mock_sysvar_data.len() {
        return Err(ProgramError::InvalidArgument);
    }

    let available_len = mock_sysvar_data.len() - offset;
    let copy_len = core::cmp::min(buffer.len(), available_len);

    buffer[..copy_len].copy_from_slice(&mock_sysvar_data[offset..offset + copy_len]);
    Ok(())
}

/// Verifies that the mock byte-copy helper (`mock_fetch_into_unchecked`) obeys
/// the same offset semantics we expect from the real `raw::fetch_into_*` API.
///
/// This is purely an internal byte-math test; it does not call the
/// production syscall wrapper and therefore does not attest that the runtime
/// offset logic works.  Its value is guarding against mistakes
/// in the offset arithmetic used by other in-test helpers.
#[test]
fn test_mock_offset_copy() {
    // Create mock sysvar data: 8-byte length + 3 entries
    let entries = &[
        (100u64, [1u8; HASH_BYTES]),
        (99u64, [2u8; HASH_BYTES]),
        (98u64, [3u8; HASH_BYTES]),
    ];
    let mock_sysvar_data = create_mock_data(entries);

    // Test offset 0 (full data)
    let mut buffer_full = std::vec![0u8; mock_sysvar_data.len()];
    mock_fetch_into_unchecked(&mock_sysvar_data, &mut buffer_full, 0).unwrap();
    assert_eq!(buffer_full, mock_sysvar_data);

    // Test offset 8 (skip length prefix, get entries only)
    let entries_size = 3 * ENTRY_SIZE;
    let mut buffer_entries = std::vec![0u8; entries_size];
    mock_fetch_into_unchecked(&mock_sysvar_data, &mut buffer_entries, 8).unwrap();
    assert_eq!(buffer_entries, &mock_sysvar_data[8..8 + entries_size]);

    // Test offset 8 + ENTRY_SIZE (skip first entry)
    let remaining_entries_size = 2 * ENTRY_SIZE;
    let mut buffer_skip_first = std::vec![0u8; remaining_entries_size];
    let skip_first_offset = 8 + ENTRY_SIZE;
    mock_fetch_into_unchecked(
        &mock_sysvar_data,
        &mut buffer_skip_first,
        skip_first_offset as u64,
    )
    .unwrap();
    assert_eq!(
        buffer_skip_first,
        &mock_sysvar_data[skip_first_offset..skip_first_offset + remaining_entries_size]
    );

    // Test partial read with small buffer
    let mut small_buffer = [0u8; 16]; // Only 16 bytes
    mock_fetch_into_unchecked(&mock_sysvar_data, &mut small_buffer, 0).unwrap();
    assert_eq!(small_buffer, &mock_sysvar_data[0..16]);

    // Test offset beyond data (should fail)
    let mut buffer_beyond = [0u8; 10];
    let beyond_offset = mock_sysvar_data.len() as u64;
    assert!(
        mock_fetch_into_unchecked(&mock_sysvar_data, &mut buffer_beyond, beyond_offset).is_err()
    );
}

#[test]
fn test_entries_exposed_no_std() {
    let entries = generate_mock_entries(8, 80, DecrementStrategy::Strictly1);
    let data = create_mock_data(&entries);
    let sh = unsafe { SlotHashes::new_unchecked(data.as_slice()) };

    let slice = sh.entries();
    assert_eq!(slice.len(), entries.len());
    for (i, e) in slice.iter().enumerate() {
        assert_eq!(e.slot(), entries[i].0);
        assert_eq!(e.hash, entries[i].1);
    }
}

#[test]
fn test_safe_vs_unsafe_getters_consistency() {
    let entries = generate_mock_entries(16, 200, DecrementStrategy::Strictly1);
    let data = create_mock_data(&entries);
    let sh = unsafe { SlotHashes::new_unchecked(data.as_slice()) };

    for i in 0..entries.len() {
        let safe_entry = sh.get_entry(i).unwrap();
        let unsafe_entry = unsafe { sh.get_entry_unchecked(i) };
        assert_eq!(safe_entry, unsafe_entry);
    }

    assert_eq!(sh.len(), entries.len());
}

#[test]
fn test_entry_count_header_too_short() {
    let short = [0u8; 4];
    assert!(SlotHashes::new(&short[..]).is_err());
    assert_eq!(read_entry_count_from_bytes(&short), None);
}
