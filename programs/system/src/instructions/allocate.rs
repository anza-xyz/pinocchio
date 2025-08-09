use pinocchio::{
    account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey, ProgramResult,
};

use crate::CanInvoke;

/// Allocate space in a (possibly new) account without funding.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` New account
pub struct Allocate<'a> {
    /// Account to be assigned.
    pub account: &'a AccountInfo,

    /// Number of bytes of memory to allocate.
    pub space: u64,
}

const ACCOUNTS_LEN: usize = 1;

impl<'a> CanInvoke for Allocate<'a> {
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
        // -  [4..12]: space
        let mut instruction_data = [0; 12];
        instruction_data[0] = 8;
        instruction_data[4..12].copy_from_slice(&self.space.to_le_bytes());

        invoke(
            &crate::ID,
            &[self.account],
            &[AccountMeta::writable_signer(self.account.key())],
            &instruction_data,
        )
    }
}
