use solana_account_view::AccountView;
use solana_instruction_view::{cpi::invoke, AccountRole, InstructionView};
use solana_program_error::ProgramResult;

/// One-time idempotent upgrade of legacy nonce versions in order to bump
/// them out of chain blockhash domain.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
pub struct UpgradeNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a AccountView,
}

impl UpgradeNonceAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // account metadata
        let account_metas: [AccountRole; 1] = [AccountRole::writable(self.account.address())];

        // instruction
        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &[12, 0, 0, 0],
        };

        invoke(&instruction, &[self.account])
    }
}
