use pinocchio::{
    cpi::{invoke_signed, Signer},
    instruction::{InstructionAccount, InstructionView},
    AccountView, ProgramResult,
};

/// Update the authority of the provided record account.
///
/// ### Accounts:
///   0. `[WRITE]` Record account, must be previously initialized
///   1. `[SIGNER]` Current record authority.
///   2. `[]` New record authority.
pub struct SetAuthority<'a> {
    /// Record account.
    pub account: &'a AccountView,
    /// Current record authority.
    pub current_authority: &'a AccountView,
    /// New record authority.
    pub new_authority: &'a AccountView,
}

impl SetAuthority<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let instruction_accounts: [InstructionAccount; 3] = [
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::readonly_signer(self.current_authority.address()),
            InstructionAccount::readonly(self.new_authority.address()),
        ];

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            data: &[2],
        };

        invoke_signed(
            &instruction,
            &[self.account, self.current_authority, self.new_authority],
            signers,
        )
    }
}
