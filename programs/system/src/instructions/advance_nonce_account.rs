use solana_account_view::AccountView;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    AccountMeta, InstructionView,
};
use solana_program_error::ProgramResult;

/// Consumes a stored nonce, replacing it with a successor.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[]` RecentBlockhashes sysvar
///   2. `[SIGNER]` Nonce authority
pub struct AdvanceNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a AccountView,

    /// RecentBlockhashes sysvar.
    pub recent_blockhashes_sysvar: &'a AccountView,

    /// Nonce authority.
    pub authority: &'a AccountView,
}

impl AdvanceNonceAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly(self.recent_blockhashes_sysvar.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // instruction
        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &[4],
        };

        invoke_signed(
            &instruction,
            &[self.account, self.recent_blockhashes_sysvar, self.authority],
            signers,
        )
    }
}
