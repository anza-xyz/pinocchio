use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    Address,
    cpi::invoke,
    instruction::{AccountMeta, Instruction},
    ProgramResult,
};

use crate::{write_bytes, UNINIT_BYTE};

/// Initialize a new Token Account.
///
/// ### Accounts:
///   0. `[WRITE]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
///   3. `[]` Rent sysvar
pub struct InitializeAccount2<'a, 'b> {
    /// New Account.
    pub account: &'a AccountInfo,
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Rent Sysvar Account
    pub rent_sysvar: &'a AccountInfo,
    /// Owner of the new Account.
    pub owner: &'a Address,
    /// Token Program
    pub token_program: &'b Address,
}

impl InitializeAccount2<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly(self.rent_sysvar.key()),
        ];

        // instruction data
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..33]: owner (32 bytes, Address)
        let mut instruction_data = [UNINIT_BYTE; 33];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[16]);
        // Set owner as [u8; 32] at offset [1..33]
        write_bytes(&mut instruction_data[1..], self.owner.as_array());

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 33) },
        };

        invoke(&instruction, &[self.account, self.mint, self.rent_sysvar])
    }
}
