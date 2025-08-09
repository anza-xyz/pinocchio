use pinocchio::{
    account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey, ProgramResult,
};

use crate::CanInvoke;

/// Withdraw funds from a nonce account.
///
/// The `u64` parameter is the lamports to withdraw, which must leave the
/// account balance above the rent exempt reserve or at zero.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[WRITE]` Recipient account
///   2. `[]` Recent blockhashes sysvar
///   3. `[]` Rent sysvar
///   4. `[SIGNER]` Nonce authority
pub struct WithdrawNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a AccountInfo,

    /// Recipient account.
    pub recipient: &'a AccountInfo,

    /// Recent blockhashes sysvar.
    pub recent_blockhashes_sysvar: &'a AccountInfo,

    /// Rent sysvar.
    pub rent_sysvar: &'a AccountInfo,

    /// Nonce authority.
    pub authority: &'a AccountInfo,

    /// Lamports to withdraw.
    ///
    /// The account balance must be left above the rent exempt reserve
    /// or at zero.
    pub lamports: u64,
}

const ACCOUNTS_LEN: usize = 5;

impl<'a> CanInvoke for WithdrawNonceAccount<'a> {
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
        let mut instruction_data = [0; 12];
        instruction_data[0] = 5;
        instruction_data[4..12].copy_from_slice(&self.lamports.to_le_bytes());

        invoke(
            &crate::ID,
            &[
                self.account,
                self.recipient,
                self.recent_blockhashes_sysvar,
                self.rent_sysvar,
                self.authority,
            ],
            &[
                AccountMeta::writable(self.account.key()),
                AccountMeta::writable(self.recipient.key()),
                AccountMeta::readonly(self.recent_blockhashes_sysvar.key()),
                AccountMeta::readonly(self.rent_sysvar.key()),
                AccountMeta::readonly_signer(self.authority.key()),
            ],
            &instruction_data,
        )
    }
}
