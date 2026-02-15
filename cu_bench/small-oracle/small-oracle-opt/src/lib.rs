#![allow(unexpected_cfgs)]
#![no_std]

use pinocchio::{
    entrypoint::lazy::{AssumeNeverDup, CheckLikeType, InstructionContext},
    lazy_program_entrypoint, no_allocator, nostd_panic_handler, ProgramResult,
};

use pinocchio_small_oracle_shared_types::{
    cast_state_data_aligned, SmallOracle, SMALL_ORACLE_VALUE_SIZE,
};

lazy_program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

#[cold]
#[inline(never)]
fn hard_exit(msg: &str) -> ! {
    panic!("{}", msg);
}

fn process_instruction(mut context: InstructionContext) -> ProgramResult {
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
    let state = unsafe { cast_state_data_aligned(state_data) };

    if !pinocchio::address::address_eq(&state.authority, authority.address()) {
        hard_exit("illegal owner");
    }

    let data = unsafe { context.instruction_data_unchecked() };
    if data.len() != SMALL_ORACLE_VALUE_SIZE {
        hard_exit("invalid instruction data");
    }

    let value = u64::from_le_bytes(data.try_into().unwrap());
    state.data = value;

    Ok(())
}
