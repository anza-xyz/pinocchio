use pinocchio::{account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey};

use crate::{InvokeParts, TruncatedInstructionData};

/// Allocate space for and assign an account at an address derived
/// from a base public key and a seed.
///
/// ### Accounts:
///   0. `[WRITE]` Allocated account
///   1. `[SIGNER]` Base account
pub struct AllocateWithSeed<'a, 'b, 'c> {
    /// Allocated account.
    pub account: &'a AccountInfo,

    /// Base account.
    ///
    /// The account matching the base Pubkey below must be provided as
    /// a signer, but may be the same as the funding account and provided
    /// as account 0.
    pub base: &'a AccountInfo,

    /// String of ASCII chars, no longer than `Pubkey::MAX_SEED_LEN`.
    pub seed: &'b str,

    /// Number of bytes of memory to allocate.
    pub space: u64,

    /// Address of program that will own the new account.
    pub owner: &'c Pubkey,
}

const N_ACCOUNTS: usize = 2;
const DATA_LEN: usize = 112;

impl<'a, 'b, 'c> From<AllocateWithSeed<'a, 'b, 'c>>
    for InvokeParts<'a, N_ACCOUNTS, TruncatedInstructionData<DATA_LEN>>
{
    fn from(value: AllocateWithSeed<'a, 'b, 'c>) -> Self {
        Self {
            program_id: crate::ID,
            accounts: [value.account, value.base],
            account_metas: [
                AccountMeta::writable_signer(value.account.key()),
                AccountMeta::readonly_signer(value.base.key()),
            ],
            instruction_data: {
                // instruction data
                // - [0..4  ]: instruction discriminator
                // - [4..36 ]: base pubkey
                // - [36..44]: seed length
                // - [44..  ]: seed (max 32)
                // - [..  +8]: account space
                // - [.. +32]: owner pubkey
                let mut instruction_data = [0; DATA_LEN];
                instruction_data[0] = 9;
                instruction_data[4..36].copy_from_slice(value.base.key());
                instruction_data[36..44]
                    .copy_from_slice(&u64::to_le_bytes(value.seed.len() as u64));

                let offset = 44 + value.seed.len();
                instruction_data[44..offset].copy_from_slice(value.seed.as_bytes());
                instruction_data[offset..offset + 8].copy_from_slice(&value.space.to_le_bytes());
                instruction_data[offset + 8..offset + 40].copy_from_slice(value.owner.as_ref());
                TruncatedInstructionData::new(instruction_data, offset + 40)
            },
        }
    }
}
