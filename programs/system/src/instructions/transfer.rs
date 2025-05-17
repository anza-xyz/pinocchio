use pinocchio::{account_info::AccountInfo, instruction::AccountMeta};

use crate::{FullInstructionData, InvokeParts};

/// Transfer lamports.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Funding account
///   1. `[WRITE]` Recipient account
pub struct Transfer<'a> {
    /// Funding account.
    pub from: &'a AccountInfo,

    /// Recipient account.
    pub to: &'a AccountInfo,

    /// Amount of lamports to transfer.
    pub lamports: u64,
}

const N_ACCOUNTS: usize = 2;
const DATA_LEN: usize = 12;

impl<'a> From<Transfer<'a>> for InvokeParts<'a, N_ACCOUNTS, FullInstructionData<DATA_LEN>> {
    fn from(value: Transfer<'a>) -> Self {
        InvokeParts {
            program_id: crate::ID,
            accounts: [value.from, value.to],
            account_metas: [
                AccountMeta::writable_signer(value.from.key()),
                AccountMeta::writable(value.to.key()),
            ],
            instruction_data: {
                // instruction data
                // -  [0..4 ]: instruction discriminator
                // -  [4..12]: lamports amount
                let mut instruction_data = [0; DATA_LEN];
                instruction_data[0] = 2;
                instruction_data[4..12].copy_from_slice(&value.lamports.to_le_bytes());
                FullInstructionData::new(instruction_data)
            },
        }
    }
}
