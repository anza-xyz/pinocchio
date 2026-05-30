use pinocchio::{
    cpi::{invoke_signed, Signer},
    instruction::{InstructionAccount, InstructionView},
    AccountView, ProgramResult,
};

/// Close the provided record account, draining lamports to recipient account.
///
/// ### Accounts:
///   0. `[WRITE]` Record account, must be previously initialized.
///   1. `[SIGNER]` Record authority.
///   2. `[]` Receiver of account lamports.
pub struct CloseAccount<'a> {
    /// Record account.
    pub account: &'a AccountView,
    /// Record authority.
    pub authority: &'a AccountView,
    /// Account lamports recipient.
    pub receiver: &'a AccountView,
}

impl CloseAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let instruction_accounts: [InstructionAccount; 3] = [
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
            InstructionAccount::readonly(self.receiver.address()),
        ];

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            data: &[3],
        };

        invoke_signed(
            &instruction,
            &[self.account, self.authority, self.receiver],
            signers,
        )
    }
}
