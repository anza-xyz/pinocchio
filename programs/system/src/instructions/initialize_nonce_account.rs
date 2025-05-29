use pinocchio::{account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey};

use crate::{FullInstructionData, InvokeParts};

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
///   1. `[]` RecentBlockhashes sysvar
///   2. `[]` Rent sysvar
pub struct InitializeNonceAccount<'a, 'b> {
    /// Nonce account.
    pub account: &'a AccountInfo,

    /// RecentBlockhashes sysvar.
    pub recent_blockhashes_sysvar: &'a AccountInfo,

    /// Rent sysvar.
    pub rent_sysvar: &'a AccountInfo,

    /// Lamports to withdraw.
    ///
    /// The account balance muat be left above the rent exempt reserve
    /// or at zero.
    pub authority: &'b Pubkey,
}

const N_ACCOUNTS: usize = 3;
const DATA_LEN: usize = 36;

impl<'a, 'b> From<InitializeNonceAccount<'a, 'b>>
    for InvokeParts<'a, N_ACCOUNTS, FullInstructionData<DATA_LEN>>
{
    fn from(value: InitializeNonceAccount<'a, 'b>) -> Self {
        InvokeParts {
            program_id: crate::ID,
            accounts: [
                value.account,
                value.recent_blockhashes_sysvar,
                value.rent_sysvar,
            ],
            account_metas: [
                AccountMeta::writable(value.account.key()),
                AccountMeta::readonly(value.recent_blockhashes_sysvar.key()),
                AccountMeta::readonly(value.rent_sysvar.key()),
            ],
            instruction_data: {
                // instruction data
                // -  [0..4 ]: instruction discriminator
                // -  [4..36]: authority pubkey
                let mut instruction_data = [0; DATA_LEN];
                instruction_data[0] = 6;
                instruction_data[4..36].copy_from_slice(value.authority);
                FullInstructionData::new(instruction_data)
            },
        }
    }
}
