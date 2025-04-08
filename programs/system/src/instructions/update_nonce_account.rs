use solana_account_view::AccountView;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    AccountMeta, InstructionView,
};
use solana_program_error::ProgramResult;

/// One-time idempotent upgrade of legacy nonce versions in order to bump
/// them out of chain blockhash domain.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
pub struct UpdateNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a AccountView,
}

impl UpdateNonceAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.account.key())];

        // instruction
        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &[12],
        };

        invoke_signed(&instruction, &[self.account], signers)
    }
}
