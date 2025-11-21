use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    ProgramResult,
};

use crate::{write_bytes, UNINIT_BYTE};
use core::slice::from_raw_parts;

/// Split a stake account into two.
///
/// ### Accounts:
///   0. `[WRITE]` Stake account.
///   1. `[WRITE]` Split stake account.
///   2. `[SIGNER]` Stake authority.
pub struct Split<'a> {
    /// Stake account.
    pub stake: &'a AccountInfo,
    /// Split stake account.
    pub split_stake: &'a AccountInfo,
    /// Stake authority.
    pub stake_authority: &'a AccountInfo,
    /// Amount to split.
    pub amount: u64,
}

impl Split<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.stake.key()),
            AccountMeta::writable(self.split_stake.key()),
            AccountMeta::readonly_signer(self.stake_authority.key()),
        ];

        // Instruction data layout (LockupArgs with Option encoding):
        // - [0]: instruction discriminator (u8) = 6
        // - [1..9]: amount (8 bytes, u64)
        let mut instruction_data = [UNINIT_BYTE; 9];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[3]);
        // Set amount as u64 at offset [1..9]
        write_bytes(&mut instruction_data[1..9], &self.amount.to_le_bytes());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed(
            &instruction,
            &[self.stake, self.split_stake, self.stake_authority],
            signers,
        )
    }
}
