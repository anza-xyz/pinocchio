use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    AccountRole, InstructionView,
};
use solana_program_error::ProgramResult;

/// Revokes the delegate's authority.
///
/// ### Accounts:
///   0. `[WRITE]` The source account.
///   1. `[SIGNER]` The source account owner.
pub struct Revoke<'a, 'b> {
    /// Source Account.
    pub source: &'a AccountView,
    ///  Source Owner Account.
    pub authority: &'a AccountView,
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
        let account_metas: [AccountRole; 2] = [
            AccountRole::writable(self.source.address()),
            AccountRole::readonly_signer(self.authority.address()),
        ];

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &account_metas,
            data: &[5],
        };

        invoke_signed(&instruction, &[self.source, self.authority], signers)
    }
}
