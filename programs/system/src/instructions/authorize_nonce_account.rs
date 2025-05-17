use pinocchio::{account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey};

use crate::{FullInstructionData, InvokeParts};

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

const N_ACCOUNTS: usize = 2;
const DATA_LEN: usize = 36;

impl<'a, 'b> From<AuthorizeNonceAccount<'a, 'b>>
    for InvokeParts<'a, N_ACCOUNTS, FullInstructionData<DATA_LEN>>
{
    fn from(value: AuthorizeNonceAccount<'a, 'b>) -> Self {
        InvokeParts {
            program_id: crate::ID,
            accounts: [value.account, value.authority],
            account_metas: [
                AccountMeta::writable(value.account.key()),
                AccountMeta::readonly_signer(value.authority.key()),
            ],
            instruction_data: {
                // instruction data
                // -  [0..4 ]: instruction discriminator
                // -  [4..12]: lamports
                let mut instruction_data = [0; DATA_LEN];
                instruction_data[0] = 7;
                instruction_data[4..36].copy_from_slice(value.new_authority);

                FullInstructionData::new(instruction_data)
            },
        }
    }
}
