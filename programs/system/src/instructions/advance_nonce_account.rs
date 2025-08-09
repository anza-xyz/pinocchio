use pinocchio::{
    account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey, ProgramResult,
};

use crate::CanInvoke;

/// Consumes a stored nonce, replacing it with a successor.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[]` Recent blockhashes sysvar
///   2. `[SIGNER]` Nonce authority
pub struct AdvanceNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a AccountInfo,

    /// Recent blockhashes sysvar.
    pub recent_blockhashes_sysvar: &'a AccountInfo,

    /// Nonce authority.
    pub authority: &'a AccountInfo,
}

const ACCOUNTS_LEN: usize = 3;

impl<'a> CanInvoke for AdvanceNonceAccount<'a> {
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
            &[self.account, self.recent_blockhashes_sysvar, self.authority],
            &[
                AccountMeta::writable(self.account.key()),
                AccountMeta::readonly(self.recent_blockhashes_sysvar.key()),
                AccountMeta::readonly_signer(self.authority.key()),
            ],
            &[4],
        )
    }
}
