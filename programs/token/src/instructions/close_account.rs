use solana_account_view::AccountView;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    AccountMeta, InstructionView,
};
use solana_program_error::ProgramResult;

/// Close an account by transferring all its SOL to the destination account.
///
/// ### Accounts:
///   0. `[WRITE]` The account to close.
///   1. `[WRITE]` The destination account.
///   2. `[SIGNER]` The account's owner.
pub struct CloseAccount<'a> {
    /// Token Account.
    pub account: &'a AccountView,
    /// Destination Account
    pub destination: &'a AccountView,
    /// Owner Account
    pub authority: &'a AccountView,
}

impl CloseAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::writable(self.destination.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &[9],
        };

        invoke_signed(
            &instruction,
            &[self.account, self.destination, self.authority],
            signers,
        )
    }
}
