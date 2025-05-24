use pinocchio::{account_info::AccountInfo, instruction::AccountMeta};

use crate::{FullInstructionData, InvokeParts};

/// One-time idempotent upgrade of legacy nonce versions in order to bump
/// them out of chain blockhash domain.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
pub struct UpdateNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a AccountInfo,
}

const N_ACCOUNTS: usize = 1;
const DATA_LEN: usize = 1;

impl<'a> From<UpdateNonceAccount<'a>>
    for InvokeParts<'a, N_ACCOUNTS, FullInstructionData<DATA_LEN>>
{
    fn from(value: UpdateNonceAccount<'a>) -> Self {
        InvokeParts {
            program_id: crate::ID,
            accounts: [value.account],
            account_metas: [AccountMeta::writable(value.account.key())],
            instruction_data: FullInstructionData::new([12]),
        }
    }
}
