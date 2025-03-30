use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

use crate::{write_bytes, UNINIT_BYTE};

/// Add new validator stake account to the pool
///
/// ### Accounts:
///   0. `[WRITE]` Stake pool
///   1. `[SIGNER]` Staker
///   2. `[WRITE]` Reserve stake account
///   3. `[]` Stake pool withdraw authority
///   4. `[WRITE]` Validator stake list storage account
///   5. `[WRITE]` Stake account to add to the pool
///   6. `[]` Validator this stake account will be delegated to
///   7. `[]` Rent sysvar
///   8. `[]` Clock sysvar
///   9. '[]' Stake history sysvar
///  10. '[]' Stake config sysvar
///  11. `[]` System program
///  12. `[]` Stake program
pub struct AddValidatorToPool<'a> {
    /// Accounts
    /// Stake Pool Account.
    pub stake_pool: &'a AccountInfo,
    /// Staker Account.
    pub staker: &'a AccountInfo,
    /// Reserve Account.
    pub reserve: &'a AccountInfo,
    /// Withdraw Account.
    pub stake_pool_withdraw: &'a AccountInfo,
    /// Validator Account.
    pub validator_list: &'a AccountInfo,
    /// Stake Account.
    pub stake: &'a AccountInfo,
    /// Validator Account.
    pub validator: &'a AccountInfo,
    /// Rent Sysvar.
    pub rent_sysvar: &'a AccountInfo,
    /// Clock Sysvar.
    pub clock_sysvar: &'a AccountInfo,
    /// Stake History Sysvar.
    pub stake_history_sysvar: &'a AccountInfo,
    /// Stake Config Sysvar.
    pub stake_config_sysvar: &'a AccountInfo,
    /// System Program.
    pub system_program: &'a AccountInfo,
    /// Stake Program.
    pub stake_program: &'a AccountInfo,

    /// input
    /// Seed.
    pub seed: Option<u32>,
}

impl AddValidatorToPool<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 13] = [
            AccountMeta::writable(self.stake_pool.key()),
            AccountMeta::readonly_signer(self.staker.key()),
            AccountMeta::writable(self.reserve.key()),
            AccountMeta::readonly(self.stake_pool_withdraw.key()),
            AccountMeta::writable(self.validator_list.key()),
            AccountMeta::writable(self.stake.key()),
            AccountMeta::readonly(self.validator.key()),
            AccountMeta::readonly(self.rent_sysvar.key()),
            AccountMeta::readonly(self.clock_sysvar.key()),
            AccountMeta::readonly(self.stake_history_sysvar.key()),
            AccountMeta::readonly(self.stake_config_sysvar.key()),
            AccountMeta::readonly(self.system_program.key()),
            AccountMeta::readonly(self.stake.key()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: seed presence flag (1 byte, u8)
        // -  [2..6]: seed (4 bytes, u32)
        let mut instruction_data = [UNINIT_BYTE; 6];

        // Set discriminator as u8 at offet [0]
        write_bytes(&mut instruction_data, &[0]);
        // Set COption & seed at offset [1..6]
        if let Some(seed_auth) = self.seed {
            write_bytes(&mut instruction_data[1..2], &[1]);
            write_bytes(&mut instruction_data[2..6], &seed_auth.to_le_bytes());
        } else {
            write_bytes(&mut instruction_data[1..2], &[0]);
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 6) },
        };

        invoke_signed(
            &instruction,
            &[
                self.stake_pool,
                self.staker,
                self.reserve,
                self.stake_pool_withdraw,
                self.validator_list,
                self.stake,
                self.validator,
                self.rent_sysvar,
                self.clock_sysvar,
                self.stake_history_sysvar,
                self.stake_config_sysvar,
                self.system_program,
                self.system_program,
                self.stake_program,
            ],
            signers,
        )
    }
}
