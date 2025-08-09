use pinocchio::{
    account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey, ProgramResult,
};

use crate::CanInvoke;

/// Transfer lamports from a derived address.
///
/// ### Accounts:
///   0. `[WRITE]` Funding account
///   1. `[SIGNER]` Base for funding account
///   2. `[WRITE]` Recipient account
pub struct TransferWithSeed<'a, 'b, 'c> {
    /// Funding account.
    pub from: &'a AccountInfo,

    /// Base account.
    ///
    /// The account matching the base Pubkey below must be provided as
    /// a signer, but may be the same as the funding account and provided
    /// as account 0.
    pub base: &'a AccountInfo,

    /// Recipient account.
    pub to: &'a AccountInfo,

    /// Amount of lamports to transfer.
    pub lamports: u64,

    /// String of ASCII chars, no longer than `Pubkey::MAX_SEED_LEN`.
    pub seed: &'b str,

    /// Address of program that will own the new account.
    pub owner: &'c Pubkey,
}

const ACCOUNTS_LEN: usize = 3;

impl<'a, 'b, 'c> CanInvoke for TransferWithSeed<'a, 'b, 'c> {
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
        // - [0..4  ]: instruction discriminator
        // - [4..12 ]: lamports amount
        // - [12..20]: seed length
        // - [20..  ]: seed (max 32)
        // - [.. +32]: owner pubkey
        let mut instruction_data = [0; 80];
        instruction_data[0] = 11;
        instruction_data[4..12].copy_from_slice(&self.lamports.to_le_bytes());
        instruction_data[12..20].copy_from_slice(&u64::to_le_bytes(self.seed.len() as u64));

        let offset = 20 + self.seed.len();
        instruction_data[20..offset].copy_from_slice(self.seed.as_bytes());
        instruction_data[offset..offset + 32].copy_from_slice(self.owner.as_ref());

        invoke(
            &crate::ID,
            &[self.from, self.base, self.to],
            &[
                AccountMeta::writable(self.from.key()),
                AccountMeta::readonly_signer(self.base.key()),
                AccountMeta::writable(self.to.key()),
            ],
            &instruction_data[..offset + 32],
        )
    }
}
