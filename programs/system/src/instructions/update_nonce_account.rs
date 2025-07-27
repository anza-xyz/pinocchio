use pinocchio::{
    account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey, ProgramResult,
};

use crate::CanInvoke;

/// One-time idempotent upgrade of legacy nonce versions in order to bump
/// them out of chain blockhash domain.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
pub struct UpdateNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a AccountInfo,
}

const ACCOUNTS_LEN: usize = 1;

impl CanInvoke<ACCOUNTS_LEN> for UpdateNonceAccount<'_> {
    fn invoke_via(
        self,
        invoke: impl FnOnce(
            /* program_id: */ &Pubkey,
            /* accounts: */ &[&AccountInfo; 1],
            /* account_metas: */ &[AccountMeta],
            /* data: */ &[u8],
        ) -> ProgramResult,
    ) -> ProgramResult {
        invoke(
            &crate::ID,
            &[self.account],
            &[AccountMeta::writable(self.account.key())],
            &[12],
        )
    }
}
