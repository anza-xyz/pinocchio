use pinocchio::{account_info::AccountInfo, instruction::AccountMeta};

use crate::{FullInstructionData, InvokeParts};

/// Consumes a stored nonce, replacing it with a successor.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[]` RecentBlockhashes sysvar
///   2. `[SIGNER]` Nonce authority
pub struct AdvanceNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a AccountInfo,

    /// RecentBlockhashes sysvar.
    pub recent_blockhashes_sysvar: &'a AccountInfo,

    /// Nonce authority.
    pub authority: &'a AccountInfo,
}

const N_ACCOUNTS: usize = 3;
const DATA_LEN: usize = 1;

impl<'a> From<AdvanceNonceAccount<'a>>
    for InvokeParts<'a, N_ACCOUNTS, FullInstructionData<DATA_LEN>>
{
    fn from(value: AdvanceNonceAccount<'a>) -> Self {
        InvokeParts {
            program_id: crate::ID,
            accounts: [
                value.account,
                value.recent_blockhashes_sysvar,
                value.authority,
            ],
            account_metas: [
                AccountMeta::writable(value.account.key()),
                AccountMeta::readonly(value.recent_blockhashes_sysvar.key()),
                AccountMeta::readonly_signer(value.authority.key()),
            ],
            instruction_data: { FullInstructionData::new([4]) },
        }
    }
}
