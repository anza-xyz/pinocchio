use pinocchio::{account_info::AccountInfo, instruction::AccountMeta};

use crate::InvokeParts;

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
const N_ACCOUNT_METAS: usize = 2;
const DATA_LEN: usize = 12;

impl<'a> InvokeParts for Transfer<'a> {
    type Accounts = [&'a AccountInfo; N_ACCOUNTS];
    type AccountMetas = [AccountMeta<'a>; N_ACCOUNT_METAS];
    type InstructionData = [u8; DATA_LEN];

    fn accounts(&self) -> Self::Accounts {
        [self.to, self.from]
    }

    fn account_metas(&self) -> Self::AccountMetas {
        [
            AccountMeta::writable_signer(self.from.key()),
            AccountMeta::writable(self.to.key()),
        ]
    }

    fn instruction_data(&self) -> (Self::InstructionData, usize) {
        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..12]: lamports amount
        let mut instruction_data = [0; DATA_LEN];
        instruction_data[0] = 2;
        instruction_data[4..12].copy_from_slice(&self.lamports.to_le_bytes());
        (instruction_data, DATA_LEN)
    }
}
