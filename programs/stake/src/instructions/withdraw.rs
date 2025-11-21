use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed_with_bounds,
    instruction::{AccountMeta, Instruction, Signer},
    ProgramResult,
};

use crate::{write_bytes, UNINIT_BYTE, UNINIT_INFO, UNINIT_META};
use core::slice::from_raw_parts;

/// Withdraw inactive lamports from a stake account.
///
/// ### Accounts:
///   0. `[WRITE]` Stake account.
///   1. `[WRITE]` Recipient account.
///   2. `[]` Clock sysvar.
///   3. `[]` Stake history sysvar.
///   4. `[SIGNER]` Withdraw authority.
///   5. `[SIGNER, OPTIONAL]` Lockup authority.
pub struct Withdraw<'a> {
    /// Stake account.
    pub stake: &'a AccountInfo,
    /// Recipient account.
    pub recipient: &'a AccountInfo,
    /// Stake authority.
    pub clock_sysvar: &'a AccountInfo,
    /// Stake history sysvar.
    pub stake_history_sysvar: &'a AccountInfo,
    /// Withdraw authority.
    pub withdraw_authority: &'a AccountInfo,
    /// Lockup authority.
    pub lockup_authority: &'a Option<AccountInfo>,
    /// Amount to withdraw.
    pub amount: u64,
}

impl Withdraw<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let mut account_metas = [UNINIT_META; 6];

        unsafe {
            // SAFETY: Always write the first 5 accounts
            account_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(self.stake.key()));
            account_metas
                .get_unchecked_mut(1)
                .write(AccountMeta::writable(self.recipient.key()));
            account_metas
                .get_unchecked_mut(2)
                .write(AccountMeta::readonly(self.clock_sysvar.key()));
            account_metas
                .get_unchecked_mut(3)
                .write(AccountMeta::readonly(self.stake_history_sysvar.key()));
            account_metas
                .get_unchecked_mut(4)
                .write(AccountMeta::readonly_signer(self.withdraw_authority.key()));

            // Write the 6th account if lockup_authority is present
            if let Some(lockup_authority) = self.lockup_authority {
                account_metas
                    .get_unchecked_mut(5)
                    .write(AccountMeta::readonly_signer(lockup_authority.key()));
            }
        }

        let num_accounts = if self.lockup_authority.is_some() {
            6
        } else {
            5
        };

        // Instruction data layout (LockupArgs with Option encoding):
        // - [0]: instruction discriminator (u8) = 6
        // - [1..9]: amount (8 bytes, u64)
        let mut instruction_data = [UNINIT_BYTE; 9];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[4]);
        // Set amount as u64 at offset [1..9]
        write_bytes(&mut instruction_data[1..9], &self.amount.to_le_bytes());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: unsafe { from_raw_parts(account_metas.as_ptr() as _, num_accounts) },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        // Account infos
        let mut account_infos = [UNINIT_INFO; 6];

        unsafe {
            account_infos.get_unchecked_mut(0).write(self.stake);
            account_infos.get_unchecked_mut(1).write(self.recipient);
            account_infos.get_unchecked_mut(2).write(self.clock_sysvar);
            account_infos
                .get_unchecked_mut(3)
                .write(self.stake_history_sysvar);
            account_infos
                .get_unchecked_mut(4)
                .write(self.withdraw_authority);
            if let Some(lockup_authority) = self.lockup_authority {
                account_infos.get_unchecked_mut(5).write(lockup_authority);
            }
        }

        invoke_signed_with_bounds::<6>(
            &instruction,
            unsafe { from_raw_parts(account_infos.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}
