use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    ProgramResult,
};

use core::mem::size_of;
use core::slice::from_raw_parts;

use crate::state::{Authorized, Lockup};
use crate::{write_bytes, UNINIT_BYTE};

/// Initialize a stake account.
///
/// ### Accounts:
///   0. `[WRITE]` Stake account.
///   1. `[]` Rent sysvar.
pub struct Initialize<'a> {
    /// Stake account.
    pub stake: &'a AccountInfo,
    /// Rent sysvar.
    pub rent_sysvar: &'a AccountInfo,
    /// Authorized.
    pub authorized: Authorized,
    /// Lockup.
    pub lockup: Lockup,
}

impl Initialize<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.stake.key()),
            AccountMeta::readonly(self.rent_sysvar.key()),
        ];

        // Instruction data
        // -  [0]   : instruction discriminator (1 byte, u8)
        // -  [1..1+size_of::<Authorized>()]: authorized (Authorized)
        // -  [1+size_of::<Authorized>()..1+size_of::<Authorized>()+size_of::<Lockup>()]: lockup (Lockup)
        let mut instruction_data = [UNINIT_BYTE; 1 + size_of::<Authorized>() + size_of::<Lockup>()];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[0]);
        // Set authorized as Authorized at offset [1..1+size_of::<Authorized>()]
        self.authorized
            .write_bytes(&mut instruction_data[1..1 + size_of::<Authorized>()]);
        // Set lockup as Lockup at offset [1+size_of::<Authorized>()..1+size_of::<Authorized>()+size_of::<Lockup>()]
        self.lockup
            .write_bytes(&mut instruction_data[1 + size_of::<Authorized>()..]);

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed(&instruction, &[self.stake, self.rent_sysvar], signers)
    }
}
