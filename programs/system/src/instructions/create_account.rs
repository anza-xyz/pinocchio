use pinocchio::{account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey};

use crate::{FullInstructionData, InvokeParts};

/// Create a new account.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Funding account
///   1. `[WRITE, SIGNER]` New account
pub struct CreateAccount<'a> {
    /// Funding account.
    pub from: &'a AccountInfo,

    /// New account.
    pub to: &'a AccountInfo,

    /// Number of lamports to transfer to the new account.
    pub lamports: u64,

    /// Number of bytes of memory to allocate.
    pub space: u64,

    /// Address of program that will own the new account.
    pub owner: &'a Pubkey,
}

const N_ACCOUNTS: usize = 2;
const DATA_LEN: usize = 52;

impl<'a> From<CreateAccount<'a>> for InvokeParts<'a, N_ACCOUNTS, FullInstructionData<DATA_LEN>> {
    fn from(value: CreateAccount<'a>) -> Self {
        InvokeParts {
            program_id: crate::ID,
            accounts: [value.from, value.to],
            account_metas: [
                AccountMeta::writable_signer(value.from.key()),
                AccountMeta::writable_signer(value.to.key()),
            ],
            instruction_data: {
                // instruction data
                // - [0..4  ]: instruction discriminator
                // - [4..12 ]: lamports
                // - [12..20]: account space
                // - [20..52]: owner pubkey
                let mut instruction_data = [0; DATA_LEN];
                // create account instruction has a '0' discriminator
                instruction_data[4..12].copy_from_slice(&value.lamports.to_le_bytes());
                instruction_data[12..20].copy_from_slice(&value.space.to_le_bytes());
                instruction_data[20..52].copy_from_slice(value.owner.as_ref());

                FullInstructionData::new(instruction_data)
            },
        }
    }
}
