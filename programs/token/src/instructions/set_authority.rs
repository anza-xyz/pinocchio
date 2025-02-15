use core::slice::from_raw_parts;

use {
    pinocchio::{
        account_info::AccountInfo,
        instruction::{AccountMeta, Instruction, Signer},
        pubkey::Pubkey,
        ProgramResult,
    },
    pinocchio_cpi::invoke_signed,
};

use crate::{write_bytes, UNINIT_BYTE};

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum AuthorityType {
    MintTokens = 0,
    FreezeAccount = 1,
    AccountOwner = 2,
    CloseAccount = 3,
}

/// Sets a new authority of a mint or account.
///
/// ### Accounts:
///   0. `[WRITE]` The mint or account to change the authority of.
///   1. `[SIGNER]` The current authority of the mint or account.
pub struct SetAuthority<'a> {
    /// Account (Mint or Token)
    pub account: &'a AccountInfo,

    /// Authority of the Account.
    pub authority: &'a AccountInfo,

    /// The type of authority to update.
    pub authority_type: AuthorityType,

    /// The new authority
    pub new_authority: Option<&'a Pubkey>,
}

impl SetAuthority<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // instruction data
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: authority_type (1 byte, u8)
        // -  [2]: new_authority presence flag (1 byte, AuthorityType)
        // -  [3..35] new_authority (optional, 32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 35];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[6]);
        // Set authority_type as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[self.authority_type as u8]);
        // Set new_authority as [u8; 32] at offset [2..35]
        if let Some(new_authority) = self.new_authority {
            write_bytes(&mut instruction_data[2..3], &[1]);
            write_bytes(&mut instruction_data[3..], new_authority);
        } else {
            write_bytes(&mut instruction_data[2..3], &[0]);
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 35) },
        };

        invoke_signed(&instruction, &[self.account, self.authority], signers)
    }
}
