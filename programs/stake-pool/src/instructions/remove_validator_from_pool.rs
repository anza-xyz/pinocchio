use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

/// Removes validator from the pool, deactivating its stake
///
/// ### Accounts:
///   0. `[w]` Stake pool
///   1. `[s]` Staker
///   2. `[]` Stake pool withdraw authority
///   3. `[w]` Validator stake list storage account
///   4. `[w]` Stake account to remove from the pool
///   5. `[w]` Transient stake account, to deactivate if necessary
///   6. `[]` Sysvar clock
///   7. `[]` Stake program id,
pub struct RemoveValidatorFromPool<'a> {
    /// Accounts
    /// Stake Pool Account.
    pub stake_pool: &'a AccountInfo,
    /// Staker Account.
    pub staker: &'a AccountInfo,
    /// Withdraw Account.
    pub stake_pool_withdraw: &'a AccountInfo,
    /// Validator Account.
    pub validator_list: &'a AccountInfo,
    /// Stake Account.
    pub stake: &'a AccountInfo,
    ///  Transient stake account.
    pub transient_stake: &'a AccountInfo,
    /// Clock Sysvar.
    pub clock_sysvar: &'a AccountInfo,
    /// Stake Program.
    pub stake_program: &'a AccountInfo,
}

impl RemoveValidatorFromPool<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 8] = [
            AccountMeta::writable(self.stake_pool.key()),
            AccountMeta::readonly_signer(self.staker.key()),
            AccountMeta::readonly(self.stake_pool_withdraw.key()),
            AccountMeta::writable(self.validator_list.key()),
            AccountMeta::writable(self.stake.key()),
            AccountMeta::writable(self.transient_stake.key()),
            AccountMeta::readonly(self.clock_sysvar.key()),
            AccountMeta::readonly(self.stake_program.key()),
        ];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &[2],
        };

        invoke_signed(
            &instruction,
            &[
                self.stake_pool,
                self.staker,
                self.stake_pool_withdraw,
                self.validator_list,
                self.stake,
                self.transient_stake,
                self.clock_sysvar,
                self.stake_program,
            ],
            signers,
        )
    }
}
