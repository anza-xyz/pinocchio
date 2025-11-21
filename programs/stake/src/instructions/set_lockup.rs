use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, UNINIT_BYTE};
use core::slice::from_raw_parts;

/// Set lockup for a stake account.
///
/// ### Accounts:
///   0. `[WRITE]` Stake account.
///   1. `[SIGNER]` Lockup authority or Withdraw authority.
pub struct SetLockup<'a, 'b> {
    /// Stake account.
    pub stake: &'a AccountInfo,
    /// Lockup authority or Withdraw authority.
    pub authority: &'a AccountInfo,
    /// Unix timestamp at which the lockup expires.
    pub unix_timestamp: Option<i64>,
    /// Epoch at which the lockup expires.
    pub epoch: Option<u64>,
    /// The custodian pubkey that can modify the lockup.
    pub custodian: Option<&'b Pubkey>,
}

impl SetLockup<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.stake.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data layout (LockupArgs with Option encoding):
        // - [0]: instruction discriminator (u8) = 6
        // - Option<i64> unix_timestamp: 1 byte tag + 8 bytes if Some
        // - Option<u64> epoch: 1 byte tag + 8 bytes if Some
        // - Option<Pubkey> custodian: 1 byte tag + 32 bytes if Some
        // Max size: 1 + (1+8) + (1+8) + (1+32) = 52 bytes
        let mut instruction_data = [UNINIT_BYTE; 52];
        let mut offset = 0;

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[offset..], &[6]);
        offset += 1;

        // Write unix_timestamp Option<i64>
        match self.unix_timestamp {
            Some(ts) => {
                let bytes = ts.to_le_bytes();
                write_bytes(
                    &mut instruction_data[offset..],
                    &[
                        1, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ],
                );
                offset += 9;
            }
            None => {
                write_bytes(&mut instruction_data[offset..], &[0]);
                offset += 1;
            }
        }

        // Write epoch Option<u64>
        match self.epoch {
            Some(e) => {
                let bytes = e.to_le_bytes();
                write_bytes(
                    &mut instruction_data[offset..],
                    &[
                        1, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ],
                );
                offset += 9;
            }
            None => {
                write_bytes(&mut instruction_data[offset..], &[0]);
                offset += 1;
            }
        }

        // Write custodian Option<Pubkey>
        match self.custodian {
            Some(c) => {
                write_bytes(&mut instruction_data[offset..], &[1]);
                write_bytes(&mut instruction_data[offset + 1..], c.as_ref());
                offset += 33;
            }
            None => {
                write_bytes(&mut instruction_data[offset..], &[0]);
                offset += 1;
            }
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, offset) },
        };

        invoke_signed(&instruction, &[self.stake, self.authority], signers)
    }
}
