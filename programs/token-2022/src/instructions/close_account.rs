use pinocchio::{
    account::AccountView,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    Address, ProgramResult,
};

/// Close an account by transferring all its SOL to the destination account.
///
/// ### Accounts:
///   0. `[WRITE]` The account to close.
///   1. `[WRITE]` The destination account.
///   2. `[SIGNER]` The account's owner.
pub struct CloseAccount<'a, 'b> {
    /// Token Account.
    pub account: &'a AccountView,
    /// Destination Account
    pub destination: &'a AccountView,
    /// Owner Account
    pub authority: &'a AccountView,
    /// Token Program
    pub token_program: &'b Address,
}

impl CloseAccount<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.account.address()),
            AccountMeta::writable(self.destination.address()),
            AccountMeta::readonly_signer(self.authority.address()),
        ];

        let instruction = Instruction {
            program_id: self.token_program,
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
