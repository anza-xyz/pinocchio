use core::slice;

use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed_with_bounds,
    instruction::{AccountMeta, Instruction, Signer},
    ProgramResult,
};

use crate::state::StakeAuthorize;
use crate::{write_bytes, UNINIT_BYTE, UNINIT_INFO, UNINIT_META};

/// Change the authority of a specific type of a stake account.
///
/// ### Accounts:
///   0. `[WRITE]` The stake account.
///   1. `[]` The clock sysvar.
///   2. `[SIGNER]` The current authority of the stake account.
///   3. `[SIGNER]` The new authority of the stake account.
///   4. `[SIGNER, OPTIONAL]` The lockup authority (or custodian) of the stake account.
pub struct AuthorizeChecked<'a> {
    /// Stake Account.
    pub stake: &'a AccountInfo,
    /// Clock Sysvar Account.
    pub clock_sysvar: &'a AccountInfo,
    /// Current Authority of the Stake Account Authority Type.
    pub authority: &'a AccountInfo,
    /// New Authority of the Stake Account.
    pub new_authority: &'a AccountInfo,
    /// Lockup Authority (or Custodian) Account.
    pub lockup_authority: &'a Option<AccountInfo>,
    /// Stake Authorize.
    pub authority_type: StakeAuthorize,
}

impl AuthorizeChecked<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let mut account_metas = [UNINIT_META; 5];

        unsafe {
            // SAFETY: Always write the first 3 accounts
            account_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(self.stake.key()));
            account_metas
                .get_unchecked_mut(1)
                .write(AccountMeta::readonly(self.clock_sysvar.key()));
            account_metas
                .get_unchecked_mut(2)
                .write(AccountMeta::readonly_signer(self.authority.key()));
            account_metas
                .get_unchecked_mut(3)
                .write(AccountMeta::readonly_signer(self.new_authority.key()));

            // Write the 4th account if lockup_authority is present
            if let Some(lockup_authority) = self.lockup_authority {
                account_metas
                    .get_unchecked_mut(4)
                    .write(AccountMeta::readonly_signer(lockup_authority.key()));
            }
        }

        let num_accounts = if self.lockup_authority.is_some() {
            5
        } else {
            4
        };

        // Instruction data
        // -  [0]   : instruction discriminator (1 byte, u8)
        // -  [1]  : authority_type (1 byte, u8)
        let mut instruction_data = [UNINIT_BYTE; 2];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[10]);
        // Set authority_type as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[self.authority_type.into()]);

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: unsafe { slice::from_raw_parts(account_metas.as_ptr() as _, num_accounts) },
            data: unsafe { slice::from_raw_parts(instruction_data.as_ptr() as _, 2) },
        };

        // Account infos
        let mut account_infos = [UNINIT_INFO; 5];

        unsafe {
            // SAFETY: Always write the first 3 accounts
            account_infos.get_unchecked_mut(0).write(self.stake);
            account_infos.get_unchecked_mut(1).write(self.clock_sysvar);
            account_infos.get_unchecked_mut(2).write(self.authority);
            account_infos.get_unchecked_mut(3).write(self.new_authority);

            // Write the 4th account if lockup_authority is present
            if let Some(lockup_authority) = self.lockup_authority {
                account_infos.get_unchecked_mut(4).write(lockup_authority);
            }
        }

        invoke_signed_with_bounds::<5>(
            &instruction,
            unsafe { slice::from_raw_parts(account_infos.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}
