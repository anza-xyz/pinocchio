use solana_account_view::AccountView;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    InstructionAccount, InstructionView,
};
use solana_program_error::ProgramResult;

/// Freeze an initialized account using the Mint's freeze authority.
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

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts
        let instruction_accounts: [InstructionAccount; 3] = [
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::readonly(self.mint.address()),
            InstructionAccount::readonly_signer(self.freeze_authority.address()),
        ];

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            data: &[10],
        };

        invoke_signed(
            &instruction,
            &[self.account, self.mint, self.freeze_authority],
            signers,
        )
    }
}
