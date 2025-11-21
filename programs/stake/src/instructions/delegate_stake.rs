use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    ProgramResult,
};

/// Delegate a stake account to a specific validator (or vote account).
///
/// ### Accounts:
///   0. `[WRITE]` The stake account.
///   1. `[]` The vote account wich the stake account will be delegated to.
///   2. `[]` Clock sysvar.
///   3. `[]` Stake History sysvar.
///   4. `[]` Unused account, formerly the stake config sysvar.
///   5. `[SIGNER]` Stake Authority of the Stake Account.
pub struct DelegateStake<'a> {
    /// Stake Account.
    pub stake: &'a AccountInfo,
    /// Vote Account wich the stake account will be delegated to.
    pub vote: &'a AccountInfo,
    /// Clock Sysvar.
    pub clock_sysvar: &'a AccountInfo,
    /// Stake History Sysvar.
    pub stake_history_sysvar: &'a AccountInfo,
    /// Unused account, formerly the stake config sysvar.
    pub unused_account: &'a AccountInfo,
    /// Stake Authority of the Stake Account.
    pub stake_authority: &'a AccountInfo,
}

impl DelegateStake<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 6] = [
            AccountMeta::writable(self.stake.key()),
            AccountMeta::readonly(self.vote.key()),
            AccountMeta::readonly(self.clock_sysvar.key()),
            AccountMeta::readonly(self.stake_history_sysvar.key()),
            AccountMeta::readonly(self.unused_account.key()),
            AccountMeta::readonly_signer(self.stake_authority.key())
        ];

        // Instruction data
        let instruction_data = [2u8; 1];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(
            &instruction,
            &[self.stake, self.vote, self.clock_sysvar, self.stake_history_sysvar, self.unused_account, self.stake_authority],
            signers,
        )
    }
}
