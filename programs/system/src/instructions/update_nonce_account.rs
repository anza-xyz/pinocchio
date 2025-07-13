use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

/// One-time idempotent upgrade of legacy nonce versions in order to bump
/// them out of chain blockhash domain.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
pub struct UpdateNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a AccountInfo,
}

impl UpdateNonceAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.account.key())];

        // instruction
        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &[12],
        };

        invoke_signed(&instruction, &[self.account], signers)
    }
}
