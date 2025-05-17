use pinocchio::{account_info::AccountInfo, instruction::AccountMeta};

use crate::{FullInstructionData, InvokeParts};

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

const N_ACCOUNTS: usize = 1;
const DATA_LEN: usize = 12;

impl<'a> From<Allocate<'a>> for InvokeParts<'a, N_ACCOUNTS, FullInstructionData<DATA_LEN>> {
    fn from(value: Allocate<'a>) -> Self {
        InvokeParts {
            program_id: crate::ID,
            accounts: [value.account],
            account_metas: [AccountMeta::writable_signer(value.account.key())],
            instruction_data: {
                // instruction data
                // -  [0..4 ]: instruction discriminator
                // -  [4..12]: space
                let mut instruction_data = [0; DATA_LEN];
                instruction_data[0] = 8;
                instruction_data[4..12].copy_from_slice(&value.space.to_le_bytes());
                FullInstructionData::new(instruction_data)
            },
        }
    }
}
