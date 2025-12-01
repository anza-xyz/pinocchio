use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    ProgramResult,
};

/// Deactivate a stake account from a delinquent vote account.
///
/// ### Accounts:
///   0. `[WRITE]` The stake account.
///   1. `[]` The delinquent vote account.
///   2. `[]` The reference vote account.
pub struct DeactivateDelinquent<'a> {
    /// Stake Account.
    pub stake: &'a AccountInfo,
    /// Delinquent Vote Account.
    pub delinquent_vote_account: &'a AccountInfo,
    /// Reference Vote Account.
    pub reference_vote_account: &'a AccountInfo,
}

impl DeactivateDelinquent<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.stake.key()),
            AccountMeta::readonly(self.delinquent_vote_account.key()),
            AccountMeta::readonly(self.reference_vote_account.key()),
        ];

        // Instruction data
        let instruction_data = 14u32.to_le_bytes();

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(
            &instruction,
            &[
                self.stake,
                self.delinquent_vote_account,
                self.reference_vote_account,
            ],
            signers,
        )
    }
}
