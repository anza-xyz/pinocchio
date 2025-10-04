use pinocchio::{
    account::AccountView,
    cpi::invoke,
    instruction::{AccountMeta, Instruction},
    ProgramResult,
};

/// Given a native token account updates its amount field based
/// on the account's underlying `lamports`.
///
/// ### Accounts:
///   0. `[WRITE]`  The native token account to sync with its underlying
///      lamports.
pub struct SyncNative<'a> {
    /// Native Token Account
    pub native_token: &'a AccountView,
}

impl SyncNative<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.native_token.address())];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &[17],
        };

        invoke(&instruction, &[self.native_token])
    }
}
