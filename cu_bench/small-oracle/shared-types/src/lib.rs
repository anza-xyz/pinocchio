#![no_std]

use pinocchio::address::{address_eq, Address};

#[repr(C)]
#[derive(Clone)]
pub struct SmallOracle {
    pub authority: Address,
    pub data: u64,
}

pub const SMALL_ORACLE_ACCOUNT_SIZE: usize = core::mem::size_of::<SmallOracle>();
pub const SMALL_ORACLE_VALUE_SIZE: usize = core::mem::size_of::<u64>();

#[inline(always)]
pub unsafe fn cast_state_data(state_data: &mut [u8]) -> &mut SmallOracle {
    &mut *(state_data.as_mut_ptr() as *mut SmallOracle)
}

#[inline(always)]
pub unsafe fn cast_state_data_aligned(state_data: &mut [u8]) -> &mut SmallOracle {
    debug_assert_eq!(state_data.len(), core::mem::size_of::<SmallOracle>());
    debug_assert_eq!(
        state_data
            .as_ptr()
            .align_offset(core::mem::align_of::<SmallOracle>()),
        0
    );
    cast_state_data(state_data)
}

#[inline(always)]
pub fn is_authority_match(state: &SmallOracle, authority: &Address) -> bool {
    address_eq(&state.authority, authority)
}
