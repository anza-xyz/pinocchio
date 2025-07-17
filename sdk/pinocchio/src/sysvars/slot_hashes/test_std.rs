//! Tests that rely on the `std` feature (host-only helpers, alloc, etc.).

use super::*;
use crate::account_info::{Account, AccountInfo};
use core::ptr;
extern crate std;
use super::test_utils::*;
use std::io::Write;
use std::vec::Vec;

#[test]
fn test_from_account_info_constructor() {
    std::io::stderr().flush().unwrap();

    const NUM_ENTRIES: usize = 3;
    const START_SLOT: u64 = 1234;

    let mock_entries = generate_mock_entries(NUM_ENTRIES, START_SLOT, DecrementStrategy::Strictly1);
    let data = create_mock_data(&mock_entries);

    let mut aligned_backing: Vec<u64>;
    #[allow(unused_assignments)]
    let mut acct_ptr: *mut Account = core::ptr::null_mut();

    unsafe {
        let header_size = core::mem::size_of::<AccountLayout>();
        let total_size = header_size + data.len();
        let word_len = (total_size + 7) / 8;
        aligned_backing = std::vec![0u64; word_len];
        let base_ptr = aligned_backing.as_mut_ptr() as *mut u8;

        let header_ptr = base_ptr as *mut AccountLayout;
        ptr::write(
            header_ptr,
            AccountLayout {
                borrow_state: crate::NON_DUP_MARKER,
                is_signer: 0,
                is_writable: 0,
                executable: 0,
                resize_delta: 0,
                key: SLOTHASHES_ID,
                owner: [0u8; 32],
                lamports: 0,
                data_len: data.len() as u64,
            },
        );

        ptr::copy_nonoverlapping(data.as_ptr(), base_ptr.add(header_size), data.len());

        acct_ptr = base_ptr as *mut Account;
    }

    let account_info = AccountInfo { raw: acct_ptr };

    let slot_hashes = SlotHashes::from_account_info(&account_info)
        .expect("from_account_info should succeed with well-formed data");

    assert_eq!(slot_hashes.len(), NUM_ENTRIES);
    for (i, entry) in slot_hashes.into_iter().enumerate() {
        assert_eq!(entry.slot(), mock_entries[i].0);
        assert_eq!(entry.hash, mock_entries[i].1);
    }
}

/// Host-side sanity test: ensure the `SlotHashes::fetch()` helper compiles and
/// allocates a MAX_SIZE-sized buffer without panicking.
///
/// On non-Solana targets the underlying syscall is stubbed; the returned buffer
/// is zero-initialized and contains zero entries.  We overwrite
/// that buffer with deterministic fixture data and then exercise the normal
/// `SlotHashes` getters to make sure the view itself works.  We do not verify
/// that the syscall populated real on-chain bytes, as doing so requires an
/// environment outside the scope of host `cargo test`.
#[cfg(feature = "std")]
#[test]
fn test_fetch_allocates_buffer_host() {
    const START_SLOT: u64 = 500;
    let entries = generate_mock_entries(5, START_SLOT, DecrementStrategy::Strictly1);
    let data = create_mock_data(&entries);

    // This should allocate a 20_488-byte boxed slice and *not* panic.
    let mut slot_hashes =
        SlotHashes::<std::boxed::Box<[u8]>>::fetch().expect("fetch() should allocate");

    // Overwrite the stubbed contents with known data so we can reuse the
    // remainder of the test harness.
    slot_hashes.data[..data.len()].copy_from_slice(&data);

    assert_eq!(slot_hashes.len(), entries.len());
    for (i, entry) in slot_hashes.into_iter().enumerate() {
        assert_eq!(entry.slot(), entries[i].0);
        assert_eq!(entry.hash, entries[i].1);
    }
}
