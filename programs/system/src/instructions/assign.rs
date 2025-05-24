use pinocchio::{account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey};

use crate::{FullInstructionData, InvokeParts};

/// Assign account to a program
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Assigned account public key
pub struct Assign<'a, 'b> {
    /// Account to be assigned.
    pub account: &'a AccountInfo,

    /// Program account to assign as owner.
    pub owner: &'b Pubkey,
}

const N_ACCOUNTS: usize = 1;
const DATA_LEN: usize = 36;

impl<'a, 'b> From<Assign<'a, 'b>> for InvokeParts<'a, N_ACCOUNTS, FullInstructionData<DATA_LEN>> {
    fn from(value: Assign<'a, 'b>) -> Self {
        InvokeParts {
            program_id: crate::ID,
            accounts: [value.account],
            account_metas: [AccountMeta::writable_signer(value.account.key())],
            instruction_data: {
                // instruction data
                // -  [0..4 ]: instruction discriminator
                // -  [4..36]: owner pubkey
                let mut instruction_data = [0; DATA_LEN];
                instruction_data[0] = 1;
                instruction_data[4..36].copy_from_slice(value.owner.as_ref());

                FullInstructionData::new(instruction_data)
            },
        }
    }
}
