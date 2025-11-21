use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    ProgramResult,
};

/// Initialize a stake account.
///
/// ### Accounts:
///   0. `[WRITE]` Stake account.
///   1. `[]` Rent sysvar.
///   2. `[]` Stake Authority.
///   3. `[SIGNER]` Withdraw Authority.
pub struct InitializeChecked<'a> {
    /// Stake account.
    pub stake: &'a AccountInfo,
    /// Rent sysvar.
    pub rent_sysvar: &'a AccountInfo,
    /// Stake Authority.
    pub stake_authority: &'a AccountInfo,
    /// Withdraw Authority.
    pub withdraw_authority: &'a AccountInfo,
}

impl InitializeChecked<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.stake.key()),
            AccountMeta::readonly(self.rent_sysvar.key()),
            AccountMeta::readonly(self.stake_authority.key()),
            AccountMeta::readonly_signer(self.withdraw_authority.key()),
        ];

        // Instruction data
        let instruction_data = [9u8; 1];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(
            &instruction,
            &[
                self.stake,
                self.rent_sysvar,
                self.stake_authority,
                self.withdraw_authority,
            ],
            signers,
        )
    }
}
