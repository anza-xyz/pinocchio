use pinocchio::{
    cpi::invoke,
    instruction::{InstructionAccount, InstructionView},
    AccountView, ProgramResult,
};

/// One-time idempotent upgrade of legacy nonce versions in order to bump
/// them out of chain blockhash domain.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
pub struct UpgradeNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a AccountView,
}

impl UpgradeNonceAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Instruction accounts
        let instruction_accounts: [InstructionAccount; 1] =
            [InstructionAccount::writable(self.account.address())];

        // instruction
        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            data: &[12, 0, 0, 0],
        };

        invoke(&instruction, &[self.account])
    }
}
