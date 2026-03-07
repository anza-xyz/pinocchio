//! Shared helpers for entrypoint tests.
//! Compiled only when `cfg(test)` is active.

use {
    super::*,
    ::alloc::{
        alloc::{alloc, dealloc, handle_alloc_error},
        vec,
        vec::Vec,
    },
    core::{alloc::Layout, ptr::copy_nonoverlapping},
};

pub const MOCK_PROGRAM_ID: Address = Address::new_from_array([5u8; 32]);

pub struct AlignedMemory {
    ptr: *mut u8,
    pub layout: Layout,
}

impl AlignedMemory {
    pub fn new(len: usize) -> Self {
        let layout = Layout::from_size_align(len, BPF_ALIGN_OF_U128).unwrap();
        unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                handle_alloc_error(layout);
            }
            AlignedMemory { ptr, layout }
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that the `data` length does not exceed the
    /// remaining space in the memory region starting from the `offset`.
    pub unsafe fn write(&mut self, data: &[u8], offset: usize) {
        copy_nonoverlapping(data.as_ptr(), self.ptr.add(offset), data.len());
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }
}

impl Drop for AlignedMemory {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr, self.layout);
        }
    }
}

pub enum AccountDesc {
    NonDup { data_len: usize },
    Dup { original_index: u8 },
}

/// Creates an input buffer with per-account control over data_len and dup status.
///
/// # Safety
///
/// The caller must pass `AccountDesc` entries consistent with what the SVM
/// loader would produce (e.g. dup indices must reference earlier accounts).
pub unsafe fn create_input_custom(
    accounts: &[AccountDesc],
    instruction_data: &[u8],
) -> AlignedMemory {
    let mut input = AlignedMemory::new(1_000_000_000);
    input.write(&(accounts.len() as u64).to_le_bytes(), 0);
    let mut offset = size_of::<u64>();

    for desc in accounts {
        match desc {
            AccountDesc::NonDup { data_len } => {
                let mut account = [0u8; STATIC_ACCOUNT_DATA + size_of::<u64>()];
                account[0] = NON_DUP_MARKER;
                account[80..88].copy_from_slice(&data_len.to_le_bytes());
                input.write(&account, offset);
                offset += account.len();
                let padding_for_data =
                    (*data_len + (BPF_ALIGN_OF_U128 - 1)) & !(BPF_ALIGN_OF_U128 - 1);
                input.write(&vec![0u8; padding_for_data], offset);
                offset += padding_for_data;
            }
            AccountDesc::Dup { original_index } => {
                input.write(&[*original_index, 0, 0, 0, 0, 0, 0, 0], offset);
                offset += size_of::<u64>();
            }
        }
    }

    input.write(&instruction_data.len().to_le_bytes(), offset);
    offset += size_of::<u64>();
    input.write(instruction_data, offset);
    offset += instruction_data.len();
    input.write(MOCK_PROGRAM_ID.as_array(), offset);

    input
}

/// Creates an input buffer where each account N has `data_len = N`.
///
/// # Safety
///
/// Same as [`create_input_custom`].
pub unsafe fn create_input(accounts: usize, instruction_data: &[u8]) -> AlignedMemory {
    let descs: Vec<_> = (0..accounts)
        .map(|i| AccountDesc::NonDup { data_len: i })
        .collect();
    create_input_custom(&descs, instruction_data)
}

/// Creates an input buffer with `accounts - duplicated` unique accounts
/// followed by `duplicated` dups of the last unique account.
///
/// # Safety
///
/// Same as [`create_input_custom`].
pub unsafe fn create_input_with_duplicates(
    accounts: usize,
    instruction_data: &[u8],
    duplicated: usize,
) -> AlignedMemory {
    if accounts == 0 {
        return create_input_custom(&[], instruction_data);
    }
    assert!(
        duplicated < accounts,
        "Duplicated accounts must be less than total accounts"
    );
    let unique = accounts - duplicated;
    let mut descs: Vec<_> = (0..unique)
        .map(|i| AccountDesc::NonDup { data_len: i })
        .collect();
    for _ in 0..duplicated {
        descs.push(AccountDesc::Dup {
            original_index: (unique - 1) as u8,
        });
    }
    create_input_custom(&descs, instruction_data)
}
