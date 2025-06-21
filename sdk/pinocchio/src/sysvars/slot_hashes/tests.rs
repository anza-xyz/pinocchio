use crate::{
    program_error::ProgramError,
    sysvars::slot_hashes::raw,
    sysvars::{clock::Slot, slot_hashes::*},
};
use core::mem::{align_of, size_of};
extern crate std;
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

fn create_mock_data(entries: &[(u64, [u8; 32])]) -> Vec<u8> {
    let num_entries = entries.len() as u64;
    let data_len = NUM_ENTRIES_SIZE + entries.len() * ENTRY_SIZE;
    let mut data = std::vec![0u8; data_len];
    data[0..NUM_ENTRIES_SIZE].copy_from_slice(&num_entries.to_le_bytes());
    let mut offset = NUM_ENTRIES_SIZE;
    for (slot, hash) in entries {
        data[offset..offset + SLOT_SIZE].copy_from_slice(&slot.to_le_bytes());
        data[offset + SLOT_SIZE..offset + ENTRY_SIZE].copy_from_slice(hash);
        offset += ENTRY_SIZE;
    }
    data
}

fn generate_mock_entries(
    num_entries: usize,
    start_slot: u64,
    strategy: DecrementStrategy,
) -> Vec<(u64, [u8; 32])> {
    let mut entries = Vec::with_capacity(num_entries);
    let mut current_slot = start_slot;
    for i in 0..num_entries {
        let hash_byte = (i % 256) as u8;
        let hash = [hash_byte; 32];
        entries.push((current_slot, hash));
        let random_val = simple_prng(i as u64);
        let decrement = match strategy {
            DecrementStrategy::Strictly1 => 1,
            DecrementStrategy::Average1_05 => {
                if random_val % 20 == 0 {
                    2
                } else {
                    1
                }
            }
            #[allow(dead_code)] // May be used by benchmarks
            DecrementStrategy::Average2 => {
                if random_val % 2 == 0 {
                    1
                } else {
                    3
                }
            }
        };
        current_slot = current_slot.saturating_sub(decrement);
    }
    entries
}

#[cfg(feature = "std")]
mod std_tests {
    use super::*;

    #[test]
    fn test_iterator_into_ref() {
        let entries = generate_mock_entries(10, 50, DecrementStrategy::Strictly1);
        let data = create_mock_data(&entries);
        let sh = unsafe { SlotHashes::new_unchecked(data.as_slice(), entries.len()) };

        let mut collected: Vec<u64> = Vec::new();
        for e in &sh {
            collected.push(e.slot());
        }
        let expected: Vec<u64> = entries.iter().map(|(s, _)| *s).collect();
        assert_eq!(collected, expected);

        let iter = (&sh).into_iter();
        assert_eq!(iter.len(), sh.len());
    }

    #[test]
    fn test_from_account_info_constructor() {
        use crate::account_info::{Account, AccountInfo};
        use crate::pubkey::Pubkey;
        use core::{mem, ptr};

        const NUM_ENTRIES: usize = 3;
        const START_SLOT: u64 = 1234;
        let mock_entries =
            generate_mock_entries(NUM_ENTRIES, START_SLOT, DecrementStrategy::Strictly1);
        let data = create_mock_data(&mock_entries);

        let mut aligned_backing: Vec<u64>;
        #[allow(unused_assignments)]
        let mut acct_ptr: *mut Account = core::ptr::null_mut();

        #[repr(C)]
        struct FakeAccount {
            borrow_state: u8,
            is_signer: u8,
            is_writable: u8,
            executable: u8,
            original_data_len: u32,
            key: Pubkey,
            owner: Pubkey,
            lamports: u64,
            data_len: u64,
        }

        unsafe {
            // Build a contiguous Vec<u8> with header followed by SlotHashes payload.
            let header_size = mem::size_of::<FakeAccount>();
            let mut blob: Vec<u8> = std::vec![0u8; header_size + data.len()];

            let header_ptr = &mut blob[0] as *mut u8 as *mut FakeAccount;
            ptr::write(
                header_ptr,
                FakeAccount {
                    borrow_state: 0,
                    is_signer: 0,
                    is_writable: 0,
                    executable: 0,
                    original_data_len: 0,
                    key: SLOTHASHES_ID,
                    owner: [0u8; 32],
                    lamports: 0,
                    data_len: data.len() as u64,
                },
            );

            ptr::copy_nonoverlapping(
                data.as_ptr(),
                blob.as_mut_ptr().add(header_size),
                data.len(),
            );

            let word_len = (blob.len() + 7) / 8;
            aligned_backing = std::vec![0u64; word_len];
            ptr::copy_nonoverlapping(
                blob.as_ptr(),
                aligned_backing.as_mut_ptr() as *mut u8,
                blob.len(),
            );

            // Purposely shadow the earlier variables so the remainder of the test
            // works unchanged.
            let ptr_u8 = aligned_backing.as_mut_ptr() as *mut u8;
            acct_ptr = ptr_u8 as *mut Account;
        }

        let account_info = AccountInfo { raw: acct_ptr };
        let slot_hashes = SlotHashes::from_account_info(&account_info)
            .expect("from_account_info should succeed with well-formed data");

        // Basic sanity checks on the returned view.
        assert_eq!(slot_hashes.len(), NUM_ENTRIES);
        for (i, entry) in slot_hashes.into_iter().enumerate() {
            assert_eq!(entry.slot(), mock_entries[i].0);
            assert_eq!(entry.hash, mock_entries[i].1);
        }
    }

    #[test]
    fn test_fetch_std_path() {
        // Mock SlotHashes data (5 entries) that we expect to observe after a
        // successful syscall on-chain.
        const START_SLOT: u64 = 500;
        let entries = generate_mock_entries(5, START_SLOT, DecrementStrategy::Strictly1);
        let data = create_mock_data(&entries);

        // Call the real fetch() implementation first. This ensures we run
        // through all internal logic (allocation, length checks, etc.). On
        // the host, the underlying syscall is a no-op so the returned
        // buffer is zero-initialised.
        let mut slot_hashes =
            SlotHashes::<Box<[u8]>>::fetch().expect("fetch() should succeed on host");

        // Back-fill the buffer with the mock payload so that all accessors
        // operate on realistic data.
        slot_hashes.data[..data.len()].copy_from_slice(&data);
        // Recompute the entry count now that the header is populated.
        slot_hashes.len = unsafe { read_entry_count_from_bytes_unchecked(&slot_hashes.data) };

        assert_eq!(slot_hashes.len(), entries.len());
        for (i, entry) in slot_hashes.into_iter().enumerate() {
            assert_eq!(entry.slot(), entries[i].0);
            assert_eq!(entry.hash, entries[i].1);
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
enum DecrementStrategy {
    Strictly1,
    Average1_05,
    Average2,
}

// Stand-in for proper fuzz (todo)
fn simple_prng(seed: u64) -> u64 {
    const A: u64 = 16807;
    const M: u64 = 2147483647;
    let initial_state = if seed == 0 { 1 } else { seed };
    (A.wrapping_mul(initial_state)) % M
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

    let slot_hashes = unsafe { SlotHashes::new_unchecked(data.as_slice(), entry_count) };

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
    let empty_hashes = unsafe { SlotHashes::new_unchecked(empty_data.as_slice(), 0) };
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
    let slot_hashes = unsafe { SlotHashes::new_unchecked(data.as_slice(), NUM_ENTRIES) };

    assert_eq!(slot_hashes.len(), NUM_ENTRIES);
    assert!(!slot_hashes.is_empty());

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
    let empty_hashes = unsafe { SlotHashes::new_unchecked(empty_data.as_slice(), 0) };
    assert_eq!(empty_hashes.len(), 0);
    assert!(empty_hashes.is_empty());
    assert!(empty_hashes.get_entry(0).is_none());
    assert!(empty_hashes.into_iter().next().is_none());
}

#[test]
fn test_get_entry_count_no_std() {
    // Valid data (2 entries)
    let entries: &[(Slot, [u8; HASH_BYTES])] = &[(100, [1u8; HASH_BYTES]), (98, [2u8; HASH_BYTES])];
    let num_entries_bytes = (entries.len() as u64).to_le_bytes();
    const TEST_LEN: usize = 2;
    let mut raw_data = [0u8; NUM_ENTRIES_SIZE + TEST_LEN * ENTRY_SIZE];
    raw_data[..NUM_ENTRIES_SIZE].copy_from_slice(&num_entries_bytes);
    let mut cursor = NUM_ENTRIES_SIZE;
    for (slot, hash) in entries {
        raw_data[cursor..cursor + SLOT_SIZE].copy_from_slice(&slot.to_le_bytes());
        cursor += SLOT_SIZE;
        raw_data[cursor..cursor + HASH_BYTES].copy_from_slice(hash.as_ref());
        cursor += HASH_BYTES;
    }
    let data_slice = &raw_data[..cursor];

    let slot_hashes = SlotHashes::new(data_slice).expect("valid data should parse");
    assert_eq!(slot_hashes.len(), 2);
    let count_res = slot_hashes.get_entry_count();
    assert!(count_res.is_ok());
    assert_eq!(count_res.unwrap(), 2);
    let count_res_unchecked = unsafe { slot_hashes.get_entry_count_unchecked() };
    assert_eq!(count_res_unchecked, 2);

    // Data too small (less than len prefix)
    let short_data_1 = &data_slice[0..NUM_ENTRIES_SIZE - 1];
    let res1 = SlotHashes::new(short_data_1);
    assert!(matches!(res1, Err(ProgramError::AccountDataTooSmall)));

    // Data too small (correct len prefix, but not enough data for entries)
    let short_data_2 = &data_slice[0..NUM_ENTRIES_SIZE + ENTRY_SIZE];
    let res2 = SlotHashes::new(short_data_2);
    assert!(matches!(res2, Err(ProgramError::InvalidArgument)));

    let count_res_unchecked_2 = unsafe { read_entry_count_from_bytes_unchecked(short_data_2) };
    assert_eq!(count_res_unchecked_2, 2);

    // Empty data is valid
    let empty_num_bytes = (0u64).to_le_bytes();
    let mut empty_raw_data = [0u8; NUM_ENTRIES_SIZE];
    empty_raw_data[..NUM_ENTRIES_SIZE].copy_from_slice(&empty_num_bytes);
    let empty_hashes =
        SlotHashes::new(empty_raw_data.as_slice()).expect("empty data should be valid");
    assert_eq!(empty_hashes.len(), 0);
    let empty_res = empty_hashes.get_entry_count();
    assert!(empty_res.is_ok());
    assert_eq!(empty_res.unwrap(), 0);
    let unsafe_empty_len = unsafe { read_entry_count_from_bytes_unchecked(&empty_raw_data) };
    assert_eq!(unsafe_empty_len, 0);
}

#[test]
fn test_get_entry_unchecked_no_std() {
    let single_entry: &[(Slot, [u8; HASH_BYTES])] = &[(100, [1u8; HASH_BYTES])];
    let num_entries_bytes_1 = (single_entry.len() as u64).to_le_bytes();
    const TEST_LEN_1: usize = 1;
    let mut raw_data_1 = [0u8; NUM_ENTRIES_SIZE + TEST_LEN_1 * ENTRY_SIZE];
    raw_data_1[..NUM_ENTRIES_SIZE].copy_from_slice(&num_entries_bytes_1);
    raw_data_1[NUM_ENTRIES_SIZE..NUM_ENTRIES_SIZE + SLOT_SIZE]
        .copy_from_slice(&single_entry[0].0.to_le_bytes());
    raw_data_1[NUM_ENTRIES_SIZE + SLOT_SIZE..].copy_from_slice(single_entry[0].1.as_ref());
    let slot_hashes = unsafe { SlotHashes::new_unchecked(raw_data_1.as_slice(), 1) };

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
    let sh = unsafe { SlotHashes::new_unchecked(data.as_slice(), COUNT) };

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
    let sh = unsafe { SlotHashes::new_unchecked(data.as_slice(), NUM) };

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

#[test]
#[should_panic(expected = "assertion failed")]
fn test_invalid_length_debug_assert() {
    let data = std::vec![0u8; 100];
    let _sh = unsafe { SlotHashes::new_unchecked(data.as_slice(), MAX_ENTRIES + 1) };
}

#[test]
#[should_panic(expected = "assertion failed")]
fn test_insufficient_data_debug_assert() {
    let data = std::vec![0u8; NUM_ENTRIES_SIZE + 10]; // Too small for 2 entries
    let _sh = unsafe { SlotHashes::new_unchecked(data.as_slice(), 2) };
}

// Tests to verify mock data helpers
#[test]
fn mock_data_max_entries_boundary() {
    let entries = generate_mock_entries(MAX_ENTRIES, 1000, DecrementStrategy::Strictly1);
    let data = create_mock_data(&entries);
    let sh = unsafe { SlotHashes::new_unchecked(data.as_slice(), MAX_ENTRIES) };
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

#[test]
fn test_offset_functionality_with_mock() {
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
    assert_eq!(buffer_entries, &mock_sysvar_data[8..]);

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
    assert_eq!(buffer_skip_first, &mock_sysvar_data[skip_first_offset..]);

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
fn test_get_entry_count_consistency_check() {
    // Create data with space for 3 entries but only populate 2
    let entries = &[
        (100u64, [1u8; HASH_BYTES]),
        (99u64, [2u8; HASH_BYTES]),
        (98u64, [3u8; HASH_BYTES]),
    ];
    let mut data = create_mock_data(entries);

    let slot_hashes = unsafe { SlotHashes::new_unchecked(data.as_slice(), 3) };
    assert_eq!(slot_hashes.get_entry_count().unwrap(), 3);

    let slot_hashes_wrong = unsafe { SlotHashes::new_unchecked(data.as_slice(), 2) };
    assert!(slot_hashes_wrong.get_entry_count().is_err());

    data[0..8].copy_from_slice(&2u64.to_le_bytes()); // Change prefix to 2
    let slot_hashes_wrong2 = unsafe { SlotHashes::new_unchecked(data.as_slice(), 3) };
    assert!(slot_hashes_wrong2.get_entry_count().is_err());

    let slot_hashes_consistent = unsafe { SlotHashes::new_unchecked(data.as_slice(), 2) };
    assert_eq!(slot_hashes_consistent.get_entry_count().unwrap(), 2);
}

#[test]
fn test_entries_exposed_no_std() {
    let entries = generate_mock_entries(8, 80, DecrementStrategy::Strictly1);
    let data = create_mock_data(&entries);
    let sh = unsafe { SlotHashes::new_unchecked(data.as_slice(), entries.len()) };

    let slice = sh.entries();
    assert_eq!(slice.len(), entries.len());
    for (i, e) in slice.iter().enumerate() {
        assert_eq!(e.slot(), entries[i].0);
        assert_eq!(e.hash, entries[i].1);
    }
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

#[cfg(test)]
mod edge_tests {
    extern crate std;
    use crate::{
        account_info::{Account, AccountInfo},
        program_error::ProgramError,
        pubkey::Pubkey,
        sysvars::slot_hashes::*,
    };
    use core::{mem, ptr};
    use std::vec::Vec;

    fn raw_slot_hashes(declared_len: u64, entries: &[(u64, [u8; HASH_BYTES])]) -> Vec<u8> {
        let mut v = Vec::with_capacity(NUM_ENTRIES_SIZE + entries.len() * ENTRY_SIZE);
        v.extend_from_slice(&declared_len.to_le_bytes());
        for (slot, hash) in entries {
            v.extend_from_slice(&slot.to_le_bytes());
            v.extend_from_slice(hash);
        }
        v
    }

    struct AccountInfoWithBacking {
        info: AccountInfo,
        _backing: std::vec::Vec<u64>,
    }

    unsafe fn account_info_with(key: Pubkey, data: &[u8]) -> AccountInfoWithBacking {
        #[repr(C)]
        struct Header {
            borrow_state: u8,
            is_signer: u8,
            is_writable: u8,
            executable: u8,
            original_data_len: u32,
            key: Pubkey,
            owner: Pubkey,
            lamports: u64,
            data_len: u64,
        }
        let hdr_len = mem::size_of::<Header>();
        let total = hdr_len + data.len();
        let words = (total + 7) / 8;
        let mut backing: std::vec::Vec<u64> = std::vec![0u64; words];
        let hdr_ptr = backing.as_mut_ptr() as *mut Header;
        ptr::write(
            hdr_ptr,
            Header {
                borrow_state: 0,
                is_signer: 0,
                is_writable: 0,
                executable: 0,
                original_data_len: 0,
                key,
                owner: [0u8; 32],
                lamports: 0,
                data_len: data.len() as u64,
            },
        );
        ptr::copy_nonoverlapping(data.as_ptr(), (hdr_ptr as *mut u8).add(hdr_len), data.len());
        AccountInfoWithBacking {
            info: AccountInfo {
                raw: hdr_ptr as *mut Account,
            },
            _backing: backing,
        }
    }

    #[test]
    fn wrong_key_from_account_info() {
        let bytes = raw_slot_hashes(0, &[]);
        let acct_with = unsafe { account_info_with([1u8; 32], &bytes) };
        assert!(matches!(
            SlotHashes::from_account_info(&acct_with.info),
            Err(ProgramError::InvalidArgument)
        ));
    }

    #[test]
    fn too_many_entries_rejected() {
        let bytes = raw_slot_hashes((MAX_ENTRIES as u64) + 1, &[]);
        assert!(matches!(
            SlotHashes::new(bytes.as_slice()),
            Err(ProgramError::InvalidArgument)
        ));
    }

    #[test]
    fn truncated_payload_rejected() {
        let entry = (123u64, [7u8; HASH_BYTES]);
        let bytes = raw_slot_hashes(2, &[entry]); // says 2 but provides 1
        assert!(matches!(
            SlotHashes::new(bytes.as_slice()),
            Err(ProgramError::InvalidArgument)
        ));
    }

    #[test]
    fn duplicate_slots_binary_search_safe() {
        let entries = &[
            (200, [0u8; HASH_BYTES]),
            (200, [1u8; HASH_BYTES]),
            (199, [2u8; HASH_BYTES]),
        ];
        let bytes = raw_slot_hashes(entries.len() as u64, entries);
        let sh = unsafe { SlotHashes::new_unchecked(&bytes[..], entries.len()) };
        let dup_pos = sh.position(200).expect("slot 200 must exist");
        assert!(
            dup_pos <= 1,
            "binary_search should return one of the duplicate indices (0 or 1)"
        );
        assert_eq!(sh.get_hash(199), Some(&entries[2].1));
    }

    #[test]
    fn zero_len_minimal_slice_iterates_empty() {
        let zero_bytes = 0u64.to_le_bytes();
        let sh = unsafe { SlotHashes::new_unchecked(&zero_bytes[..], 0) };
        assert_eq!(sh.len(), 0);
        assert!(sh.into_iter().next().is_none());
    }
}
