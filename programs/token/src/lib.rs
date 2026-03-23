#![no_std]

#[cfg(feature = "batch")]
extern crate alloc;

//mod error;
//mod instruction;
pub mod instructions;
pub mod state;

//pub use {error::*, instruction::*};

use {
    core::mem::MaybeUninit,
    solana_instruction_view::{cpi::CpiAccount, InstructionAccount},
};

solana_address::declare_id!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

const UNINIT_CPI_ACCOUNT: MaybeUninit<CpiAccount> = MaybeUninit::<CpiAccount>::uninit();

const UNINIT_INSTRUCTION_ACCOUNT: MaybeUninit<InstructionAccount> =
    MaybeUninit::<InstructionAccount>::uninit();

#[inline(always)]
fn write_bytes(destination: &mut [MaybeUninit<u8>], source: &[u8]) {
    let len = destination.len().min(source.len());
    // SAFETY:
    // - Both pointers have alignment 1.
    // - For valid (non-UB) references, the borrow checker guarantees no overlap.
    // - `len` is bounded by both slice lengths.
    unsafe {
        core::ptr::copy_nonoverlapping(source.as_ptr(), destination.as_mut_ptr() as *mut u8, len);
    }
}
