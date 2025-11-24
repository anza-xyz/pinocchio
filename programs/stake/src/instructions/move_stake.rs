use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    ProgramResult,
};

use crate::{write_bytes, UNINIT_BYTE};
use core::slice::from_raw_parts;

/// Move stake between two stake accounts.
///
/// ### Accounts:
///   0. `[WRITE]` Source stake account.
///   1. `[WRITE]` Destination stake account.
///   2. `[SIGNER]` Stake Authority.
pub struct MoveStake<'a> {
    /// Active source stake account
    pub source_stake: &'a AccountInfo,
    /// Active or inactive destination stake account
    pub destination_stake: &'a AccountInfo,
    /// Stake Authority.
    pub stake_authority: &'a AccountInfo,
    /// Amount of stake to move.
    pub stake: u64,
}

impl MoveStake<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.destination_stake.key()),
            AccountMeta::writable(self.source_stake.key()),
            AccountMeta::readonly_signer(self.stake_authority.key()),
        ];

        // Instruction data
        // -  [0..4]   : instruction discriminator (4 bytes, u32)
        // -  [4..12]: stake amount
        let mut instruction_data = [UNINIT_BYTE; 12];
        // Set discriminator as u32 at offset [0..4]
        write_bytes(&mut instruction_data, &16u32.to_le_bytes());
        // Set stake amount as u64 at offset [4..12]
        write_bytes(&mut instruction_data[4..12], &self.stake.to_le_bytes());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed(
            &instruction,
            &[
                self.source_stake,
                self.destination_stake,
                self.stake_authority,
            ],
            signers,
        )
    }
}
