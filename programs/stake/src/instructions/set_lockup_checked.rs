use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed_with_bounds,
    instruction::{AccountMeta, Instruction, Signer},
    ProgramResult,
};

use crate::{write_bytes, UNINIT_BYTE, UNINIT_INFO, UNINIT_META};
use core::slice::from_raw_parts;

/// Set lockup for a stake account.
///
/// ### Accounts:
///   0. `[WRITE]` Stake account.
///   1. `[SIGNER]` Lockup authority or custodian.
///   2. `[SIGNER, OPTIONAL]` New Lockup authority.
pub struct SetLockupChecked<'a> {
    /// Stake account.
    pub stake: &'a AccountInfo,
    /// Lockup authority or custodian.
    pub authority: &'a AccountInfo,
    /// New Lockup authority.
    pub new_authority: Option<&'a AccountInfo>,
    /// Unix timestamp at which the lockup expires.
    pub unix_timestamp: Option<i64>,
    /// Epoch at which the lockup expires.
    pub epoch: Option<u64>,
}

impl SetLockupChecked<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let mut account_metas = [UNINIT_META; 3];

        unsafe {
            // SAFETY: Always write the first 3 accounts
            account_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(self.stake.key()));
            account_metas
                .get_unchecked_mut(1)
                .write(AccountMeta::readonly_signer(self.authority.key()));

            // Write the 4th account if new_authority is present
            if let Some(new_authority) = self.new_authority {
                account_metas
                    .get_unchecked_mut(2)
                    .write(AccountMeta::readonly_signer(new_authority.key()));
            }
        }

        let num_accounts = if self.new_authority.is_some() { 3 } else { 2 };

        // Instruction data layout (LockupArgs with Option encoding):
        // - [0]: instruction discriminator (u8) = 6
        // - Option<i64> unix_timestamp: 1 byte tag + 8 bytes if Some
        // - Option<u64> epoch: 1 byte tag + 8 bytes if Some
        // Max size: 1 + (1+8) + (1+8) = 19 bytes
        let mut instruction_data = [UNINIT_BYTE; 19];
        let mut offset = 0;

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[offset..], &[12]);
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

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: unsafe { from_raw_parts(account_metas.as_ptr() as _, num_accounts) },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, offset) },
        };

        let mut account_infos = [UNINIT_INFO; 3];

        unsafe {
            account_infos.get_unchecked_mut(0).write(self.stake);
            account_infos.get_unchecked_mut(1).write(self.authority);
            if let Some(new_authority) = self.new_authority {
                account_infos.get_unchecked_mut(2).write(new_authority);
            }
        }

        invoke_signed_with_bounds::<3>(
            &instruction,
            unsafe { from_raw_parts(account_infos.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}
