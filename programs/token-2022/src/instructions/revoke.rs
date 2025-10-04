use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    Address, ProgramResult,
};

/// Revokes the delegate's authority.
///
/// ### Accounts:
///   0. `[WRITE]` The source account.
///   1. `[SIGNER]` The source account owner.
pub struct Revoke<'a, 'b> {
    /// Source Account.
    pub source: &'a AccountInfo,
    ///  Source Owner Account.
    pub authority: &'a AccountInfo,
    /// Token Program
    pub token_program: &'b Address,
}

impl Revoke<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.source.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: &[5],
        };

        invoke_signed(&instruction, &[self.source, self.authority], signers)
    }
}
