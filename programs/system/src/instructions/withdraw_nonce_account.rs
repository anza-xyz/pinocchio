use pinocchio::{account_info::AccountInfo, instruction::AccountMeta};

use crate::{FullInstructionData, InvokeParts};

/// Withdraw funds from a nonce account.
///
/// The `u64` parameter is the lamports to withdraw, which must leave the
/// account balance above the rent exempt reserve or at zero.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[WRITE]` Recipient account
///   2. `[]` RecentBlockhashes sysvar
///   3. `[]` Rent sysvar
///   4. `[SIGNER]` Nonce authority
pub struct WithdrawNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a AccountInfo,

    /// Recipient account.
    pub recipient: &'a AccountInfo,

    /// RecentBlockhashes sysvar.
    pub recent_blockhashes_sysvar: &'a AccountInfo,

    /// Rent sysvar.
    pub rent_sysvar: &'a AccountInfo,

    /// Nonce authority.
    pub authority: &'a AccountInfo,

    /// Lamports to withdraw.
    ///
    /// The account balance muat be left above the rent exempt reserve
    /// or at zero.
    pub lamports: u64,
}

const N_ACCOUNTS: usize = 5;
const DATA_LEN: usize = 12;

impl<'a> From<WithdrawNonceAccount<'a>>
    for InvokeParts<'a, N_ACCOUNTS, FullInstructionData<DATA_LEN>>
{
    fn from(value: WithdrawNonceAccount<'a>) -> Self {
        InvokeParts {
            program_id: crate::ID,
            accounts: [
                value.account,
                value.recipient,
                value.recent_blockhashes_sysvar,
                value.rent_sysvar,
                value.authority,
            ],
            account_metas: [
                AccountMeta::writable(value.account.key()),
                AccountMeta::writable(value.recipient.key()),
                AccountMeta::readonly(value.recent_blockhashes_sysvar.key()),
                AccountMeta::readonly(value.rent_sysvar.key()),
                AccountMeta::readonly_signer(value.authority.key()),
            ],
            instruction_data: {
                // instruction data
                // -  [0..4 ]: instruction discriminator
                // -  [4..12]: lamports
                let mut instruction_data = [0; DATA_LEN];
                instruction_data[0] = 5;
                instruction_data[4..12].copy_from_slice(&value.lamports.to_le_bytes());

                FullInstructionData::new(instruction_data)
            },
        }
    }
}
