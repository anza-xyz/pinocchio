#![no_std]

pub mod instructions;
pub mod state;

use {
    core::mem::MaybeUninit, solana_account_view::AccountView,
    solana_instruction_view::InstructionAccount,
};

solana_address::declare_id!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

const AE_CIPHERTEXT_LEN: usize = 36;
const ELGAMAL_CIPHERTEXT_LEN: usize = 64;

const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

const UNINIT_ACCOUNT_REF: MaybeUninit<&AccountView> = MaybeUninit::<&AccountView>::uninit();

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
