use pinocchio::{
    cpi::invoke_signed,
    instruction::{Instruction, Signer, AccountMeta},
    account_info::AccountInfo,
    ProgramResult,
};

use core::mem::size_of;
use core::slice::from_raw_parts;

use crate::state::{Authorized, Lockup};
use crate::{write_bytes,UNINIT_BYTE};

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
        let mut instruction_data = [UNINIT_BYTE; 1 + size_of::<Authorized>() + size_of::<Lockup>()];

        write_bytes(&mut instruction_data, &[13]);
        write_bytes(&mut instruction_data[1..], &self.authorized.to_bytes());
        write_bytes(&mut instruction_data[1 + size_of::<Authorized>()..], &self.lockup.to_bytes());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &[],
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed(
            &instruction,
            &[self.stake, self.rent_sysvar],
            signers,
        )
    }
}
