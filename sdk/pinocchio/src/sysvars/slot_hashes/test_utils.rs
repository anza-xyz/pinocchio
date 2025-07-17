//! Shared helpers for SlotHashes sysvar tests.
//! This module is compiled only when `cfg(test)` is active so `std` can be used
//! freely while production code remains `#![no_std]`.

use super::*;
extern crate std;
use crate::account_info::{Account, AccountInfo};
use crate::pubkey::Pubkey;
use core::{mem, ptr};
use std::vec::Vec;

/// Matches the pinocchio Account struct.
/// Account fields are private, so this struct allows more readable
/// use of them in tests.
#[repr(C)]
#[derive(Clone, Copy)]
struct AccountLayout {
    borrow_state: u8,
    is_signer: u8,
    is_writable: u8,
    executable: u8,
    resize_delta: i32,
    key: Pubkey,
    owner: Pubkey,
    lamports: u64,
    data_len: u64,
}

/// Strategy that decides how much the slot number is decremented between
/// successive entries in `generate_mock_entries`.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum DecrementStrategy {
    /// Always decrement by exactly 1.
    Strictly1,
    /// Mostly ‑1 with occasional ‑2 so that the *average* decrement ≈ 1.05.
    Average1_05,
    /// Roughly 50 % chance of ‑1 and 50 % chance of ‑3 (average ≈ 2).
    Average2,
}

/// Tiny deterministic PRNG (linear-congruential) good enough for unit tests.
#[inline]
pub fn simple_prng(seed: u64) -> u64 {
    const A: u64 = 16_807;
    const M: u64 = 2_147_483_647; // 2^31 ‑ 1
    let s = if seed == 0 { 1 } else { seed };
    (A.wrapping_mul(s)) % M
}

/// Produce `num_entries` mock `(slot, hash)` pairs sorted by slot descending.
pub fn generate_mock_entries(
    num_entries: usize,
    start_slot: u64,
    strategy: DecrementStrategy,
) -> Vec<(u64, [u8; HASH_BYTES])> {
    let mut entries = Vec::with_capacity(num_entries);
    let mut current_slot = start_slot;
    for i in 0..num_entries {
        let hash_byte = (i % 256) as u8;
        let hash = [hash_byte; HASH_BYTES];
        entries.push((current_slot, hash));

        let random_val = simple_prng(i as u64);
        let dec = match strategy {
            DecrementStrategy::Strictly1 => 1,
            DecrementStrategy::Average1_05 => {
                if random_val % 20 == 0 {
                    2
                } else {
                    1
                }
            }
            DecrementStrategy::Average2 => {
                if random_val % 2 == 0 {
                    1
                } else {
                    3
                }
            }
        };
        current_slot = current_slot.saturating_sub(dec);
    }
    entries
}

/// Build a `Vec<u8>` the size of the *golden* SlotHashes sysvar (20 488 bytes)
/// containing the supplied `entries` and with the `declared_len` header.
pub fn build_slot_hashes_bytes(declared_len: u64, entries: &[(u64, [u8; HASH_BYTES])]) -> Vec<u8> {
    let mut data = std::vec![0u8; MAX_SIZE];
    data[..NUM_ENTRIES_SIZE].copy_from_slice(&declared_len.to_le_bytes());
    let mut offset = NUM_ENTRIES_SIZE;
    for (slot, hash) in entries {
        data[offset..offset + SLOT_SIZE].copy_from_slice(&slot.to_le_bytes());
        data[offset + SLOT_SIZE..offset + ENTRY_SIZE].copy_from_slice(hash);
        offset += ENTRY_SIZE;
    }
    data
}

/// Convenience wrapper where `declared_len == entries.len()`.
#[inline]
pub fn create_mock_data(entries: &[(u64, [u8; HASH_BYTES])]) -> Vec<u8> {
    build_slot_hashes_bytes(entries.len() as u64, entries)
}

/// Allocate a heap-backed `AccountInfo` whose data region is initialized with
/// `data` and whose key is `key`.
///
/// The function also returns the backing `Vec<u64>` so the caller can keep it
/// alive for the duration of the test (otherwise the memory would be freed and
/// the raw pointer inside `AccountInfo` would dangle).
///
/// # Safety
/// The caller must ensure the returned `AccountInfo` is used only for reading
/// or according to borrow rules because the Solana runtime invariants are not
/// fully enforced in this hand-rolled representation.
pub unsafe fn make_account_info(
    key: Pubkey,
    data: &[u8],
    borrow_state: u8,
) -> (AccountInfo, Vec<u64>) {
    let hdr_size = mem::size_of::<AccountLayout>();
    let total = hdr_size + data.len();
    let words = (total + 7) / 8;
    let mut backing: Vec<u64> = std::vec![0u64; words];

    let hdr_ptr = backing.as_mut_ptr() as *mut AccountLayout;
    ptr::write(
        hdr_ptr,
        AccountLayout {
            borrow_state,
            is_signer: 0,
            is_writable: 0,
            executable: 0,
            resize_delta: 0,
            key,
            owner: [0u8; 32],
            lamports: 0,
            data_len: data.len() as u64,
        },
    );

    ptr::copy_nonoverlapping(
        data.as_ptr(),
        (hdr_ptr as *mut u8).add(hdr_size),
        data.len(),
    );

    (
        AccountInfo {
            raw: hdr_ptr as *mut Account,
        },
        backing,
    )
}

#[cfg(test)]
#[test]
fn test_account_layout_compatibility() {
    assert_eq!(
        mem::size_of::<AccountLayout>(),
        mem::size_of::<Account>(),
        "Header size must match Account size"
    );
    assert_eq!(
        mem::align_of::<AccountLayout>(),
        mem::align_of::<Account>(),
        "Header alignment must match Account alignment"
    );

    unsafe {
        let test_header = AccountLayout {
            borrow_state: 42,
            is_signer: 1,
            is_writable: 1,
            executable: 0,
            resize_delta: 100,
            key: [1u8; 32],
            owner: [2u8; 32],
            lamports: 1000,
            data_len: 256,
        };

        let account_ptr = &test_header as *const AccountLayout as *const Account;
        let account_ref = &*account_ptr;
        assert_eq!(
            account_ref.borrow_state, 42,
            "borrow_state field should be accessible and match"
        );
        assert_eq!(
            account_ref.data_len, 256,
            "data_len field should be accessible and match"
        );
    }
}
