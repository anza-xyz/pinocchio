//! Syscall functions.

#[cfg(target_feature = "static-syscalls")]
pub use solana_define_syscall::sys_hash;
pub use solana_define_syscall::{define_syscall, definitions::*};

use crate::{
    instruction::{AccountMeta, ProcessedSiblingInstruction},
    pubkey::Pubkey,
};

define_syscall!(fn sol_get_return_data(data: *mut u8, length: u64, program_id: *mut Pubkey) -> u64);
define_syscall!(fn sol_get_processed_sibling_instruction(index: u64, meta: *mut ProcessedSiblingInstruction, program_id: *mut Pubkey, data: *mut u8, accounts: *mut AccountMeta) -> u64);
