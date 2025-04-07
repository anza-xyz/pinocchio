use core::slice::from_raw_parts;

use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    AccountMeta, InstructionView,
};
use solana_program_error::ProgramResult;

use crate::{write_bytes, UNINIT_BYTE};

/// Initialize a new Token Account.
///
/// ### Accounts:
///   0. `[WRITE]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
///   3. `[]` Rent sysvar
pub struct InitializeAccount2<'a> {
    /// New Account.
    pub account: &'a AccountView,
    /// Mint Account.
    pub mint: &'a AccountView,
    /// Rent Sysvar Account
    pub rent_sysvar: &'a AccountView,
    /// Owner of the new Account.
    pub owner: &'a Address,
}

impl InitializeAccount2<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
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
        write_bytes(&mut instruction_data[1..], self.owner);

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 33) },
        };

        invoke_signed(
            &instruction,
            &[self.account, self.mint, self.rent_sysvar],
            signers,
        )
    }
}
