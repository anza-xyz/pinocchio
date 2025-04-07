use solana_account_view::AccountView;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    AccountRole, InstructionView,
};
use solana_program_error::ProgramResult;

/// Consumes a stored nonce, replacing it with a successor.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[]` Recent blockhashes sysvar
///   2. `[SIGNER]` Nonce authority
pub struct AdvanceNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a AccountView,

    /// Recent blockhashes sysvar.
    pub recent_blockhashes_sysvar: &'a AccountView,

    /// Nonce authority.
    pub authority: &'a AccountView,
}

impl AdvanceNonceAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountRole; 3] = [
            AccountRole::writable(self.account.address()),
            AccountRole::readonly(self.recent_blockhashes_sysvar.address()),
            AccountRole::readonly_signer(self.authority.address()),
        ];

        // instruction
        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &[4, 0, 0, 0],
        };

        invoke_signed(
            &instruction,
            &[self.account, self.recent_blockhashes_sysvar, self.authority],
            signers,
        )
    }
}
