use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

/// Deactivate an address lookup table, making it unusable and
/// eligible for closure after a short period of time.
///
/// # Account references
///   0. `[WRITE]` Address lookup table account to deactivate
///   1. `[SIGNER]` Current authority
pub struct Deactivate<'a> {
    /// Address lookup table account to deactivate
    pub lookup_table: &'a AccountInfo,
    /// Current authority
    pub authority: &'a AccountInfo,
}

impl Deactivate<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.lookup_table.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &[3],
        };

        invoke_signed(&instruction, &[self.lookup_table, self.authority], signers)
    }
}
