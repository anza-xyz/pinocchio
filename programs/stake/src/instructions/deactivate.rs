use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    ProgramResult,
};

/// Deactivate an active stake account.
///
/// ### Accounts:
///   0. `[WRITE]` The stake account.
///   1. `[]` The clock sysvar.
///   2. `[SIGNER]` The stake authority of the stake account.
pub struct Authorize<'a> {
    /// Stake Account.
    pub stake: &'a AccountInfo,
    /// Clock Sysvar Account.
    pub clock_sysvar: &'a AccountInfo,
    /// Stake Authority of the Stake Account.
    pub stake_authority: &'a AccountInfo,
}

impl Authorize<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.stake.key()),
            AccountMeta::readonly(self.clock_sysvar.key()),
            AccountMeta::readonly_signer(self.stake_authority.key()),
        ];

        // Instruction data
        let instruction_data = [5u8; 1];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(
            &instruction,
            &[self.stake, self.clock_sysvar, self.stake_authority],
            signers,
        )
    }
}
