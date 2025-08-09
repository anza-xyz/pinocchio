use pinocchio::{
    account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey, ProgramResult,
};

use crate::CanInvoke;

/// Assign account to a program
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Assigned account public key
pub struct Assign<'a, 'b> {
    /// Account to be assigned.
    pub account: &'a AccountInfo,

    /// Program account to assign as owner.
    pub owner: &'b Pubkey,
}

const ACCOUNTS_LEN: usize = 1;

impl<'a, 'b> CanInvoke for Assign<'a, 'b> {
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
        // -  [4..36]: owner pubkey
        let mut instruction_data = [0; 36];
        instruction_data[0] = 1;
        instruction_data[4..36].copy_from_slice(self.owner.as_ref());

        invoke(
            &crate::ID,
            &[self.account],
            &[AccountMeta::writable_signer(self.account.key())],
            &instruction_data,
        )
    }
}
