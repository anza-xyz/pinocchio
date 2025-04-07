use solana_account_view::AccountView;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    AccountMeta, InstructionView,
};
use solana_program_error::ProgramResult;

/// Freeze an Initialized account using the Mint's freeze_authority
///
/// ### Accounts:
///   0. `[WRITE]` The account to freeze.
///   1. `[]` The token mint.
///   2. `[SIGNER]` The mint freeze authority.
pub struct FreezeAccount<'a> {
    /// Token Account to freeze.
    pub account: &'a AccountView,
    /// Mint Account.
    pub mint: &'a AccountView,
    /// Mint Freeze Authority Account
    pub freeze_authority: &'a AccountView,
}

impl FreezeAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly_signer(self.freeze_authority.key()),
        ];

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &[10],
        };

        invoke_signed(
            &instruction,
            &[self.account, self.mint, self.freeze_authority],
            signers,
        )
    }
}
