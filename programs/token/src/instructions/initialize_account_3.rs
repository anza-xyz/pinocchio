use core::slice::from_raw_parts;

use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::{cpi::invoke, AccountRole, InstructionView};
use solana_program_error::ProgramResult;

use crate::{write_bytes, UNINIT_BYTE};

/// Initialize a new Token Account.
///
/// ### Accounts:
///   0. `[WRITE]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
pub struct InitializeAccount3<'a> {
    /// New Account.
    pub account: &'a AccountView,
    /// Mint Account.
    pub mint: &'a AccountView,
    /// Owner of the new Account.
    pub owner: &'a Address,
}

impl InitializeAccount3<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // account metadata
        let account_metas: [AccountRole; 2] = [
            AccountRole::writable(self.account.address()),
            AccountRole::readonly(self.mint.address()),
        ];

        // instruction data
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..33]: owner (32 bytes, Address)
        let mut instruction_data = [UNINIT_BYTE; 33];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[18]);
        // Set owner as [u8; 32] at offset [1..33]
        write_bytes(&mut instruction_data[1..], self.owner.as_array());

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 33) },
        };

        invoke(&instruction, &[self.account, self.mint])
    }
}
