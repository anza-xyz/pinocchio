#![allow(unused_imports, unexpected_cfgs, dead_code)]
#![no_std]

use bytemuck::{Pod, Zeroable};
use pinocchio::{
    address::{address_eq, Address},
    entrypoint::{
        lazy::{AssumeNeverDup, CheckLikeType, InstructionContext as LazyInstructionContext},
        InstructionContext, MaybeAccount,
    },
    error::ProgramError,
    lazy_program_entrypoint, no_allocator, nostd_panic_handler, ProgramResult,
};

#[repr(C)]
#[derive(Clone, Pod, Zeroable, Copy)]
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
fn is_authority_match(state: &SmallOracle, authority: &Address) -> bool {
    address_eq(&state.authority, authority)
}

lazy_program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

#[cold]
#[inline(never)]
fn hard_exit(msg: &str) -> ! {
    panic!("{}", msg);
}

#[cfg(feature = "opt")]
fn process_instruction(mut context: LazyInstructionContext) -> ProgramResult {
    if context.remaining() != 2 {
        hard_exit("expected exactly 2 accounts");
    }

    let Ok(authority) = (unsafe {
        context.next_account_guarded(&AssumeNeverDup::new(), &CheckLikeType::<()>::new())
    }) else {
        hard_exit("Bad authority account");
    };

    if !authority.is_signer() {
        hard_exit("missing required signature");
    }

    let Ok(state) = (unsafe {
        context.next_account_guarded(&AssumeNeverDup::new(), &CheckLikeType::<SmallOracle>::new())
    }) else {
        hard_exit("Bad state account");
    };

    let state_data = unsafe { state.borrow_unchecked_mut() };
    let state: &mut SmallOracle = bytemuck::from_bytes_mut(state_data);

    if !address_eq(&state.authority, authority.address()) {
        hard_exit("illegal owner");
    }

    let data = unsafe { context.instruction_data_unchecked() };

    let value = *bytemuck::from_bytes::<u64>(data);
    state.data = value;

    Ok(())
}

#[cfg(feature = "naive")]
fn process_instruction(mut context: InstructionContext) -> ProgramResult {
    if context.remaining() != 2 {
        hard_exit("expected exactly 2 accounts");
    }

    let authority = context.next_account()?.assume_account();

    if authority.data_len() != 0 {
        hard_exit("invalid authority account data");
    }

    if !authority.is_signer() {
        hard_exit("missing required signature");
    }

    let state = context.next_account()?.assume_account();
    let state_data = unsafe { state.borrow_unchecked_mut() };

    if state_data.len() != SMALL_ORACLE_ACCOUNT_SIZE {
        hard_exit("invalid account data");
    }

    let state_ref = unsafe { cast_state_data(state_data) };

    if !is_authority_match(state_ref, authority.address()) {
        hard_exit("illegal owner");
    }

    let data = context.instruction_data()?;
    if data.len() != SMALL_ORACLE_VALUE_SIZE {
        hard_exit("invalid instruction data");
    }

    let value = u64::from_le_bytes(data.try_into().unwrap());
    state_ref.data = value;

    Ok(())
}

#[cfg(feature = "manual")]
fn process_instruction(mut context: InstructionContext) -> ProgramResult {
    if context.remaining() != 2 {
        hard_exit("expected exactly 2 accounts");
    }

    let authority = match context.next_account() {
        Ok(MaybeAccount::Account(acc)) => acc,
        Ok(MaybeAccount::Duplicated(_)) => unsafe { core::hint::unreachable_unchecked() },
        Err(_) => hard_exit("Bad authority account"),
    };

    if authority.data_len() != 0 {
        unsafe { core::hint::unreachable_unchecked() }
    }

    if !authority.is_signer() {
        hard_exit("missing required signature");
    }

    let state = match context.next_account() {
        Ok(MaybeAccount::Account(acc)) => acc,
        Ok(MaybeAccount::Duplicated(_)) => unsafe { core::hint::unreachable_unchecked() },
        Err(_) => hard_exit("Bad state account"),
    };

    let state_data = unsafe { state.borrow_unchecked_mut() };
    if state_data.len() != SMALL_ORACLE_ACCOUNT_SIZE {
        unsafe { core::hint::unreachable_unchecked() }
    }

    let state_ref = unsafe { cast_state_data(state_data) };

    if !is_authority_match(state_ref, authority.address()) {
        hard_exit("illegal owner");
    }

    let data = unsafe { context.instruction_data_unchecked() };
    if data.len() != SMALL_ORACLE_VALUE_SIZE {
        hard_exit("invalid instruction data");
    }

    let value = u64::from_le_bytes(data.try_into().unwrap());
    state_ref.data = value;

    Ok(())
}
