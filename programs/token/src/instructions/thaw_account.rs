use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

use super::TokenProgramVariant;

/// Thaw a Frozen account using the Mint's freeze_authority
///
/// ### Accounts:
///   0. `[WRITE]` The account to thaw.
///   1. `[]` The token mint.
///   2. `[SIGNER]` The mint freeze authority.
pub struct ThawAccount<'a> {
    /// Token Account to thaw.
    pub account: &'a AccountInfo,
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Mint Freeze Authority Account
    pub freeze_authority: &'a AccountInfo,
}

impl ThawAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self, token_program: TokenProgramVariant) -> ProgramResult {
        self.invoke_signed(&[], token_program)
    }

    pub fn invoke_signed(
        &self,
        signers: &[Signer],
        token_program: TokenProgramVariant,
    ) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly_signer(self.freeze_authority.key()),
        ];

        let instruction = Instruction {
            program_id: &token_program.into(),
            accounts: &account_metas,
            data: &[11],
        };

        invoke_signed(
            &instruction,
            &[self.account, self.mint, self.freeze_authority],
            signers,
        )
    }
}
