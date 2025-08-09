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

impl<'a> CanInvoke for UpdateNonceAccount<'a> {
    type Accounts = [&'a AccountInfo; ACCOUNTS_LEN];

    fn invoke_via(
        &self,
        invoke: impl FnOnce(
            /* program_id: */ &Pubkey,
            /* accounts: */ &Self::Accounts,
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
