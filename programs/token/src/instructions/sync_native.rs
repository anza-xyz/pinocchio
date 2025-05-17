use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

use crate::InstructionData;

/// Given a native token account updates its amount field based
/// on the account's underlying `lamports`.
///
/// ### Accounts:
///   0. `[WRITE]`  The native token account to sync with its underlying
///      lamports.
pub struct SyncNative<'a> {
    /// Native Token Account
    pub native_token: &'a AccountInfo,
}

impl SyncNative<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.native_token.key())];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: self.get_instruction_data(),
        };

        invoke_signed(&instruction, &[self.native_token], signers)
    }
}

impl InstructionData for SyncNative<'_> {
    #[inline]
    fn get_instruction_data(&self) -> &[u8] {
        &[17]
    }
}
