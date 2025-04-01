use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke,
    instruction::{AccountMeta, Instruction},
    Address, Address, ProgramResult,
};

/// Given a native token account updates its amount field based
/// on the account's underlying `lamports`.
///
/// ### Accounts:
///   0. `[WRITE]`  The native token account to sync with its underlying
///      lamports.
pub struct SyncNative<'a, 'b> {
    /// Native Token Account
    pub native_token: &'a AccountInfo,
    /// Token Program
    pub token_program: &'b Address,
}

impl SyncNative<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.native_token.key())];

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: &[17],
        };

        invoke(&instruction, &[self.native_token])
    }
}
