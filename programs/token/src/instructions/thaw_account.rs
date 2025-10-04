use pinocchio::{
    account_view::AccountView,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

/// Thaw a frozen account using the Mint's freeze authority.
///
/// ### Accounts:
///   0. `[WRITE]` The account to thaw.
///   1. `[]` The token mint.
///   2. `[SIGNER]` The mint freeze authority.
pub struct ThawAccount<'a> {
    /// Token Account to thaw.
    pub account: &'a AccountView,
    /// Mint Account.
    pub mint: &'a AccountView,
    /// Mint Freeze Authority Account
    pub freeze_authority: &'a AccountView,
}

impl ThawAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.account.address()),
            AccountMeta::readonly(self.mint.address()),
            AccountMeta::readonly_signer(self.freeze_authority.address()),
        ];

        let instruction = Instruction {
            program_id: &crate::ID,
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
