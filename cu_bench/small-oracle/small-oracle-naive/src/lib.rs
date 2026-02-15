#![allow(unexpected_cfgs)]
#![no_std]

use pinocchio::{
    entrypoint::InstructionContext, error::ProgramError, lazy_program_entrypoint, no_allocator,
    nostd_panic_handler, ProgramResult,
};

use pinocchio_small_oracle_shared_types::{
    cast_state_data, is_authority_match, SMALL_ORACLE_ACCOUNT_SIZE, SMALL_ORACLE_VALUE_SIZE,
};

lazy_program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

fn process_instruction(mut context: InstructionContext) -> ProgramResult {
    if context.remaining() != 2 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let authority = context.next_account()?.assume_account();

    if authority.data_len() != 0 {
        return Err(ProgramError::InvalidAccountData);
    }

    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let state = context.next_account()?.assume_account();
    let state_data = unsafe { state.borrow_unchecked_mut() };

    if state_data.len() != SMALL_ORACLE_ACCOUNT_SIZE {
        return Err(ProgramError::InvalidAccountData);
    }

    let state_ref = unsafe { cast_state_data(state_data) };

    if !is_authority_match(state_ref, authority.address()) {
        return Err(ProgramError::IllegalOwner);
    }

    let data = context.instruction_data()?;

    if data.len() != SMALL_ORACLE_VALUE_SIZE {
        return Err(ProgramError::InvalidInstructionData);
    }

    let value = u64::from_le_bytes(data.try_into().unwrap());
    state_ref.data = value;

    Ok(())
}
