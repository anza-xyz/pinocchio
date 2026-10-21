use pinocchio::{
    cpi::invoke_signed,
    instruction::{InstructionAccount, InstructionView},
    AccountView, ProgramResult,
};

/// Create a new record.
///
/// ### Accounts:
///   0. `[WRITE]` Record account, must be uninitialized
///   1. `[]` Record authority
pub struct Initialize<'a> {
    /// Record account.
    pub account: &'a AccountView,
    /// Record authority.
    pub authority: &'a AccountView,
}

impl Initialize<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let instruction_accounts: [InstructionAccount; 2] = [
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::readonly(self.authority.address()),
        ];

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            data: &[0],
        };

        invoke_signed(&instruction, &[self.account, self.authority], &[])
    }
}
