#![allow(unexpected_cfgs)]
#![no_std]

use pinocchio::{
    entrypoint::{InstructionContext, MaybeAccount},
    lazy_program_entrypoint, no_allocator, nostd_panic_handler, ProgramResult,
};

use pinocchio_small_oracle_shared_types::{
    cast_state_data, is_authority_match, SMALL_ORACLE_ACCOUNT_SIZE, SMALL_ORACLE_VALUE_SIZE,
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
