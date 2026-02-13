#![no_std]

use pinocchio::{
    address::{address_eq, Address},
    entrypoint::InstructionContext,
    error::ProgramError,
    lazy_program_entrypoint, no_allocator, nostd_panic_handler, ProgramResult,
};

lazy_program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

// should make bytemuck just as a demo that you can use plain bytemuck and NOT sacrifice sanity

#[repr(C)]
pub struct SmallOracle {
    pub authority: Address,
    pub data: u64,
}

// todo overall we can likely get better codegen by allowing a form of
// exit with a code - this requires nightly features and can be feature gated

// we will soon use bytemuck to decode bytes. this will do alignment checks
// we should inform pinochio of alignment checks

fn process_instruction(mut context: InstructionContext) -> ProgramResult {
    if context.remaining() != 2 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    // Listing many todo optimizations

    // 1. we can assume this is a non-duplicated account. should make this a feasible check
    let authority = context.next_account()?.assume_account();

    // this enable many future optimizations.
    // first, it tells downstream what size the account is. so we can assert the size
    // this check can be pushed up higher by optimizer and be used to statically know
    // that code only progresses if the size is zero, so just assume zero. BUT
    // we might need ways to propagate static checks through the calls

    if authority.data_len() != 0 {
        return Err(ProgramError::InvalidAccountData);
    }

    // required check
    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // here, we should both assume it is non-dup and check the size
    // if it's dup, then we have two sized accounts, and can't write any bogus data due to chain level restrictions
    // so just assume non-dup.
    // we should assert the size for safety, but also, optimizes computing start of instruction data

    // it need to be ensured that all of these are in place, by inspecting asm for any excess data

    let state = context.next_account()?.assume_account();

    let state_data = unsafe { state.borrow_unchecked_mut() };

    if state_data.len() != core::mem::size_of::<SmallOracle>() {
        return Err(ProgramError::InvalidAccountData);
    }

    let state_ref = unsafe { &mut *(state_data.as_mut_ptr() as *mut SmallOracle) };

    if !address_eq(&state_ref.authority, authority.address()) {
        return Err(ProgramError::IllegalOwner);
    }

    let data = context.instruction_data()?;

    // at this point it's just a copy
    if data.len() != core::mem::size_of::<u64>() {
        return Err(ProgramError::InvalidInstructionData);
    }

    // totally optimized to nothing in theory
    let value = u64::from_le_bytes(data.try_into().unwrap());

    state_ref.data = value;

    Ok(())
}
