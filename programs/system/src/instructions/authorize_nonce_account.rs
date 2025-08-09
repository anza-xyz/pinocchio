use pinocchio::{
    account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey, ProgramResult,
};

use crate::CanInvoke;

/// Change the entity authorized to execute nonce instructions on the account.
///
/// The `Pubkey` parameter identifies the entity to authorize.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[SIGNER]` Nonce authority
pub struct AuthorizeNonceAccount<'a, 'b> {
    /// Nonce account.
    pub account: &'a AccountInfo,

    /// Nonce authority.
    pub authority: &'a AccountInfo,

    /// New entity authorized to execute nonce instructions on the account.
    pub new_authority: &'b Pubkey,
}

const ACCOUNTS_LEN: usize = 2;

impl<'a, 'b> CanInvoke for AuthorizeNonceAccount<'a, 'b> {
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
        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..12]: lamports
        let mut instruction_data = [0; 36];
        instruction_data[0] = 7;
        instruction_data[4..36].copy_from_slice(self.new_authority);

        invoke(
            &crate::ID,
            &[self.account, self.authority],
            &[
                AccountMeta::writable(self.account.key()),
                AccountMeta::readonly_signer(self.authority.key()),
            ],
            &instruction_data,
        )
    }
}
