use pinocchio::{account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey};

use crate::{InvokeParts, TruncatedInstructionData};

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

const N_ACCOUNTS: usize = 3;
const DATA_LEN: usize = 80;

impl<'a, 'b, 'c> From<TransferWithSeed<'a, 'b, 'c>>
    for InvokeParts<'a, N_ACCOUNTS, TruncatedInstructionData<DATA_LEN>>
{
    fn from(value: TransferWithSeed<'a, 'b, 'c>) -> Self {
        InvokeParts {
            program_id: crate::ID,
            accounts: [value.from, value.base, value.to],
            account_metas: [
                AccountMeta::writable(value.from.key()),
                AccountMeta::readonly_signer(value.base.key()),
                AccountMeta::writable(value.to.key()),
            ],
            instruction_data: {
                // instruction data
                // - [0..4  ]: instruction discriminator
                // - [4..12 ]: lamports amount
                // - [12..20]: seed length
                // - [20..  ]: seed (max 32)
                // - [.. +32]: owner pubkey
                let mut instruction_data = [0; DATA_LEN];
                instruction_data[0] = 11;
                instruction_data[4..12].copy_from_slice(&value.lamports.to_le_bytes());
                instruction_data[12..20]
                    .copy_from_slice(&u64::to_le_bytes(value.seed.len() as u64));

                let offset = 20 + value.seed.len();
                instruction_data[20..offset].copy_from_slice(value.seed.as_bytes());
                instruction_data[offset..offset + 32].copy_from_slice(value.owner.as_ref());

                TruncatedInstructionData::new(instruction_data, offset + 32)
            },
        }
    }
}
