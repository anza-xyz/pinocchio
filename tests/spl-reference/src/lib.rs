//! Shared helpers for the tests that compare Pinocchio's token state readers
//! with the SPL reference crates.
//!
//! The tests live in `tests/`; this crate is not published.

use {
    core::{
        mem::size_of,
        ptr::{copy_nonoverlapping, slice_from_raw_parts_mut},
        slice,
    },
    solana_account_view::{AccountView, RuntimeAccount, NOT_BORROWED},
    solana_address::Address,
};

/// An account laid out in memory the way the runtime lays it out, so that
/// the `from_account_view*` constructors can be called on it.
pub struct TestAccount {
    raw: *mut RuntimeAccount,
    words: usize,
    data_len: usize,
}

impl TestAccount {
    pub fn new(owner: Address, data: &[u8]) -> Self {
        let header = size_of::<RuntimeAccount>();
        let words = (header + data.len()).div_ceil(size_of::<u64>());
        let raw = Box::into_raw(vec![0u64; words].into_boxed_slice()) as *mut RuntimeAccount;

        // SAFETY: the allocation is 8-byte aligned and large enough for the
        // header followed by `data`.
        unsafe {
            raw.write(RuntimeAccount {
                borrow_state: NOT_BORROWED,
                is_signer: 0,
                is_writable: 1,
                executable: 0,
                padding: [0; 4],
                address: Address::new_from_array([42; 32]),
                owner,
                lamports: 1,
                data_len: data.len() as u64,
            });
            copy_nonoverlapping(data.as_ptr(), (raw as *mut u8).add(header), data.len());
        }

        Self {
            raw,
            words,
            data_len: data.len(),
        }
    }

    pub fn view(&self) -> AccountView {
        // SAFETY: `raw` points to a valid `RuntimeAccount` followed by its
        // data, and stays allocated for as long as `self` lives.
        unsafe { AccountView::new_unchecked(self.raw) }
    }

    /// A copy of the account data as it currently stands.
    pub fn data(&self) -> Vec<u8> {
        // SAFETY: as in `view`; callers do not hold a borrow of the data.
        unsafe {
            slice::from_raw_parts(
                (self.raw as *const u8).add(size_of::<RuntimeAccount>()),
                self.data_len,
            )
        }
        .to_vec()
    }
}

impl Drop for TestAccount {
    fn drop(&mut self) {
        // SAFETY: `raw` came from a boxed slice of `words` words.
        drop(unsafe { Box::from_raw(slice_from_raw_parts_mut(self.raw as *mut u64, self.words)) });
    }
}
