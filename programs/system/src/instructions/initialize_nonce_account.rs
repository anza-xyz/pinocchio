use pinocchio::{
    account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey, ProgramResult,
};

use crate::CanInvoke;

/// Drive state of Uninitialized nonce account to Initialized, setting the nonce value.
///
/// The `Pubkey` parameter specifies the entity authorized to execute nonce
/// instruction on the account
///
/// No signatures are required to execute this instruction, enabling derived
/// nonce account addresses.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[]` Recent blockhashes sysvar
///   2. `[]` Rent sysvar
pub struct InitializeNonceAccount<'a, 'b> {
    /// Nonce account.
    pub account: &'a AccountInfo,

    /// Recent blockhashes sysvar.
    pub recent_blockhashes_sysvar: &'a AccountInfo,

    /// Rent sysvar.
    pub rent_sysvar: &'a AccountInfo,

    /// Lamports to withdraw.
    ///
    /// The account balance must be left above the rent exempt reserve
    /// or at zero.
    pub authority: &'b Pubkey,
}

const ACCOUNTS_LEN: usize = 3;

impl<'a, 'b> CanInvoke for InitializeNonceAccount<'a, 'b> {
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
        // -  [4..36]: authority pubkey
        let mut instruction_data = [0; 36];
        instruction_data[0] = 6;
        instruction_data[4..36].copy_from_slice(self.authority);

        invoke(
            &crate::ID,
            &[
                self.account,
                self.recent_blockhashes_sysvar,
                self.rent_sysvar,
            ],
            &[
                AccountMeta::writable(self.account.key()),
                AccountMeta::readonly(self.recent_blockhashes_sysvar.key()),
                AccountMeta::readonly(self.rent_sysvar.key()),
            ],
            &instruction_data,
        )
    }
}
