use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

/// Permanently freeze an address lookup table, making it immutable.
///
/// # Account references
///   0. `[WRITE]` Address lookup table account to freeze
///   1. `[SIGNER]` Current authority
pub struct Freeze<'a> {
    /// Address lookup table account to freeze
    pub lookup_table: &'a AccountInfo,
    /// Current authority
    pub authority: &'a AccountInfo,
}

impl Freeze<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.lookup_table.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &[1],
        };

        invoke_signed(&instruction, &[self.lookup_table, self.authority], signers)
    }
}
