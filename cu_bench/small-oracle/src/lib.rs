#![no_std]

use pinocchio::{
    entrypoint::lazy::{AssumeNeverDup, CheckLikeType, InstructionContext},
    lazy_program_entrypoint, no_allocator, nostd_panic_handler, ProgramResult,
};

use solana_address::{address_eq, Address};

lazy_program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

// Demonstrates using bytemuck access patterns with explicit runtime checks unchanged.

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
struct SmallOracleWire {
    pub authority: Address,
    pub data: u64,
}

#[inline(always)]
unsafe fn pod_from_bytes_mut_aligned<T: bytemuck::Pod>(bytes: &mut [u8]) -> &mut T {
    debug_assert_eq!(bytes.len(), core::mem::size_of::<T>());
    debug_assert_eq!(bytes.as_ptr().align_offset(core::mem::align_of::<T>()), 0);
    &mut *(bytes.as_mut_ptr() as *mut T)
}

// Keeps the hot path separate for codegen to avoid inflating CU usage on success paths.
#[cold]
#[inline(never)]
fn hard_exit(msg: &str) -> ! {
    panic!("{}", msg);
}

fn process_instruction(mut context: InstructionContext) -> ProgramResult {
    if context.remaining() != 2 {
        hard_exit("expected exactly 2 accounts");
    }

    // Can assume never dup since first account
    let Ok(authority) = (unsafe {
        context.next_account_guarded(&AssumeNeverDup::new(), &CheckLikeType::<()>::new())
    }) else {
        hard_exit("Bad authority account");
    };

    if !authority.is_signer() {
        hard_exit("missing required signature");
    }

    // Can assume not duplicate because this program expects exactly two accounts.
    let Ok(state) = (unsafe {
        context.next_account_guarded(
            &AssumeNeverDup::new(),
            &CheckLikeType::<SmallOracleWire>::new(),
        )
    }) else {
        hard_exit("Bad state account");
    };

    let state_data = unsafe { state.borrow_unchecked_mut() };
    // SAFETY: `CheckLikeType::<SmallOracleWire>` validated the byte length.
    // The SVM input buffer is 8-byte aligned and account payloads are laid out
    // on that boundary, so this cast is aligned for `SmallOracleWire` (align 8).
    let state = unsafe { pod_from_bytes_mut_aligned::<SmallOracleWire>(state_data) };

    if !address_eq(&state.authority, authority.address()) {
        hard_exit("illegal owner");
    }

    // SAFETY: both account reads have been consumed above, so instruction data is now valid.
    let data = unsafe { context.instruction_data_unchecked() };

    if data.len() != core::mem::size_of::<u64>() {
        hard_exit("invalid instruction data");
    }

    let value = bytemuck::pod_read_unaligned::<u64>(data);
    state.data = value;

    Ok(())
}
