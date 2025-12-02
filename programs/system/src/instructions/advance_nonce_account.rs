use pinocchio::{
    cpi::{invoke_signed, Signer},
    instruction::{InstructionAccount, InstructionView},
    AccountView, ProgramResult,
};

/// Consumes a stored nonce, replacing it with a successor.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[]` Recent blockhashes sysvar
///   2. `[SIGNER]` Nonce authority
pub struct AdvanceNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a AccountView,

    /// Recent blockhashes sysvar.
    pub recent_blockhashes_sysvar: &'a AccountView,

    /// Nonce authority.
    pub authority: &'a AccountView,
}

impl AdvanceNonceAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts
        let instruction_accounts: [InstructionAccount; 3] = [
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::readonly(self.recent_blockhashes_sysvar.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ];

        // instruction
        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            data: &[4, 0, 0, 0],
        };

        invoke_signed(
            &instruction,
            &[self.account, self.recent_blockhashes_sysvar, self.authority],
            signers,
        )
    }
}
