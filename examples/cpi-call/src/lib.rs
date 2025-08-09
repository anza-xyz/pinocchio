#![no_std]

use pinocchio::{program_error::ProgramError, pubkey::Pubkey};
use pinocchio_system::{callback::Invoke, instructions::Transfer};

use pinocchio::{no_allocator, nostd_panic_handler, program_entrypoint};

program_entrypoint!(crate::dispatch);
nostd_panic_handler!();
no_allocator!();

pub fn dispatch<'info>(
    _program_id: &Pubkey,
    accounts: &'info [pinocchio::account_info::AccountInfo],
    _payload: &[u8],
) -> Result<(), ProgramError> {
    Transfer {
        from: &accounts[0],
        to: &accounts[1],
        lamports: 1_000_000_000,
    }
    .invoke()
}
