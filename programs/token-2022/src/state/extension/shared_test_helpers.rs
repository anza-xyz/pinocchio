extern crate alloc;

use {
    super::{ExtensionType, ACCOUNT_TYPE_INDEX, TLV_START_INDEX},
    crate::state::AccountType,
    alloc::{vec, vec::Vec},
    core::{mem::size_of, ptr::copy_nonoverlapping},
    solana_account_view::{AccountView, RuntimeAccount, NOT_BORROWED},
    solana_address::Address,
};

pub(super) fn push_tlv_entry(buffer: &mut Vec<u8>, extension_type: ExtensionType, value: &[u8]) {
    buffer.extend_from_slice(&(extension_type as u16).to_le_bytes());
    buffer.extend_from_slice(&(value.len() as u16).to_le_bytes());
    buffer.extend_from_slice(value);
}

pub(super) fn build_mint_data(tlv_data: &[u8]) -> Vec<u8> {
    let mut data = vec![0u8; TLV_START_INDEX + tlv_data.len()];
    data[ACCOUNT_TYPE_INDEX] = AccountType::Mint as u8;
    data[TLV_START_INDEX..].copy_from_slice(tlv_data);
    data
}

pub(super) fn build_account_view(owner: &Address, data: &[u8]) -> (Vec<u64>, AccountView) {
    let runtime_len = size_of::<RuntimeAccount>();
    let total_len = runtime_len + data.len();
    let backing_len = total_len.div_ceil(size_of::<u64>());
    let mut backing = vec![0u64; backing_len];
    let raw = backing.as_mut_ptr() as *mut RuntimeAccount;

    unsafe {
        (*raw).borrow_state = NOT_BORROWED;
        (*raw).is_signer = 0;
        (*raw).is_writable = 1;
        (*raw).executable = 0;
        (*raw).padding = [0; 4];
        (*raw).address = Address::new_from_array([42u8; 32]);
        (*raw).owner = owner.clone();
        (*raw).lamports = 1;
        (*raw).data_len = data.len() as u64;

        let data_ptr = (raw as *mut u8).add(runtime_len);
        copy_nonoverlapping(data.as_ptr(), data_ptr, data.len());

        (backing, AccountView::new_unchecked(raw))
    }
}
