#![no_std]

use pinocchio::{
    address::{address_eq, Address},
    entrypoint::{AssumeNeverDup, AssumeSize, InstructionContext},
    error::ProgramError,
    lazy_program_entrypoint, no_allocator, nostd_panic_handler, ProgramResult,
};

lazy_program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

// should make bytemuck just as a demo that you can use plain bytemuck and NOT sacrifice cus+sanity

#[repr(C)]
pub struct SmallOracle {
    pub authority: Address,
    pub data: u64,
}

// cuts CUs since llvm doesn't "helpfully" optimize codegen for the cold paths
#[cold]
#[inline(never)]
// could make a version that does a bpf exit on nightly, is this worth being an
// optional pinocchio feature?
fn hard_exit(msg: &str) -> ! {
    // I think this won't spend a ton of CUs formatting the message
    panic!("{}", msg);
}

fn process_instruction(mut context: InstructionContext) -> ProgramResult {
    if context.remaining() != 2 {
        hard_exit("expected exactly 2 accounts");
    }

    let authority =
        unsafe { context.next_account_guarded(&AssumeNeverDup::new(), &AssumeSize::<0>::new()) }?;

    if !authority.is_signer() {
        hard_exit("missing required signature");
    }

    let state = unsafe {
        context.next_account_guarded(
            &AssumeNeverDup::new(),
            &AssumeSize::<{ core::mem::size_of::<SmallOracle>() }>::new(),
        )
    }?;

    let state_data = unsafe { state.borrow_unchecked_mut() };

    let state_ref = unsafe { &mut *(state_data.as_mut_ptr() as *mut SmallOracle) };

    if !address_eq(&state_ref.authority, authority.address()) {
        hard_exit("illegal owner");
    }

    let data = context.instruction_data()?;

    if data.len() != core::mem::size_of::<u64>() {
        hard_exit("invalid instruction data");
    }

    let value = u64::from_le_bytes(data.try_into().unwrap());

    state_ref.data = value;

    Ok(())
}
