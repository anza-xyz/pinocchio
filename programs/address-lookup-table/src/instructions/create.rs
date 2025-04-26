use core::slice::from_raw_parts;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    sysvars::clock::Slot,
    ProgramResult,
};

use crate::{write_bytes, UNINIT_BYTE};

/// Create an address lookup table
///
/// # Account references
///   0. `[WRITE]` Uninitialized address lookup table account
///   1. `[SIGNER]` Account used to derive and control the new address
///      lookup table.
///   2. `[SIGNER, WRITE]` Account that will fund the new address lookup
///      table.
///   3. `[]` System program for CPI.
pub struct Create<'a> {
    /// Uninitialized address lookup table account
    pub address: &'a AccountInfo,
    /// Account used to derive and control the new address lookup table.
    pub account: &'a AccountInfo,
    ///  Account that will fund the new address lookup
    pub funding_account: &'a AccountInfo,
    /// System program for CPI.
    pub system_program: &'a AccountInfo,
    /// A recent slot must be used in the derivation path
    /// for each initialized table. When closing table accounts,
    /// the initialization slot must no longer be "recent" to prevent
    /// address tables from being recreated with reordered or
    /// otherwise malicious addresses.
    recent_slot: Slot,
    /// Address tables are always initialized at program-derived
    /// addresses using the funding address, recent blockhash, and
    /// the user-passed `bump_seed`.
    bump_seed: u8,
}

impl Create<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.address.key()),
            AccountMeta::readonly_signer(self.account.key()),
            AccountMeta::writable_signer(self.funding_account.key()),
            AccountMeta::readonly(self.system_program.key()),
        ];

        // Instruction data:
        // - [0..4 ]: Instruction discriminator (4 bytes, u32) (0 for Create)
        // - [4..12]: Recent Slot
        // - [12   ]: bump seed
        let mut instruction_data = [UNINIT_BYTE; 13];
        write_bytes(&mut instruction_data, &[0]);
        write_bytes(
            &mut instruction_data[4..12],
            &self.recent_slot.to_le_bytes(),
        );
        write_bytes(&mut instruction_data[12..], &[self.bump_seed]);

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 13) },
        };

        invoke_signed(
            &instruction,
            &[
                self.address,
                self.account,
                self.funding_account,
                self.system_program,
            ],
            signers,
        )
    }
}
