use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    ProgramResult,
};

/// Merge two stake accounts.
///
/// ### Accounts:
///   0. `[WRITE]` Destination stake account.
///   1. `[WRITE]` Source stake account.
///   2. `[]` Clock sysvar.
///   3. `[]` Stake History sysvar.
///   4. `[SIGNER]` Stake Authority.
pub struct Merge<'a> {
    /// Destination stake account.
    pub destination_stake: &'a AccountInfo,
    /// Source stake account.
    pub source_stake: &'a AccountInfo,
    /// Clock sysvar.
    pub clock_sysvar: &'a AccountInfo,
    /// Stake History sysvar.
    pub stake_history_sysvar: &'a AccountInfo,
    /// Stake Authority.
    pub stake_authority: &'a AccountInfo,
}

impl Merge<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 5] = [
            AccountMeta::writable(self.destination_stake.key()),
            AccountMeta::writable(self.source_stake.key()),
            AccountMeta::readonly(self.clock_sysvar.key()),
            AccountMeta::readonly(self.stake_history_sysvar.key()),
            AccountMeta::readonly_signer(self.stake_authority.key()),
        ];

        // Instruction data
        let instruction_data = 7u32.to_le_bytes();

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(
            &instruction,
            &[
                self.destination_stake,
                self.source_stake,
                self.clock_sysvar,
                self.stake_history_sysvar,
                self.stake_authority,
            ],
            signers,
        )
    }
}
