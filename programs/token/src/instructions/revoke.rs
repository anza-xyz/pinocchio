use pinocchio::{
    account::AccountView,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

/// Revokes the delegate's authority.
///
/// ### Accounts:
///   0. `[WRITE]` The source account.
///   1. `[SIGNER]` The source account owner.
pub struct Revoke<'a> {
    /// Source Account.
    pub source: &'a AccountView,
    ///  Source Owner Account.
    pub authority: &'a AccountView,
}

impl Revoke<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.source.address()),
            AccountMeta::readonly_signer(self.authority.address()),
        ];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &[5],
        };

        invoke_signed(&instruction, &[self.source, self.authority], signers)
    }
}
