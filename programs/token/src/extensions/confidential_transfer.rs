use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

use super::ElagamalPubkey;

// Instructions

/// Initialize a new mint for a confidential transfer.
pub struct InitializeMint<'a> {
    pub mint: &'a AccountInfo,
    /// Authority to modify the `ConfidentialTransferMint` configuration and to
    /// approve new accounts.
    pub authority: Option<&'a Pubkey>,
    /// Determines if newly configured accounts must be approved by the
    /// `authority` before they may be used by the user.
    pub auto_approve_new_accounts: bool,
    /// New authority to decode any transfer amount in a confidential transfer.
    pub auditor_elgamal_pubkey: Option<&'a ElagamalPubkey>,
}

impl<'a> InitializeMint<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        let mut instruction_data = [UNINIT_BYTE; 1];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[0]);

        write_bytes(
            &mut instruction_data[1..2],
            &[self.auto_approve_new_accounts as u8],
        );

        if let Some(authority) = self.authority {
            // Set authority as Pubkey at offset [2..34]
            write_bytes(&mut instruction_data[2..3], &[1]);
            write_bytes(&mut instruction_data[2..34], authority);
        } else {
            write_bytes(&mut instruction_data[2..3], &[0]);
        }

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 1) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}

pub struct UpdateMint<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// `ConfidentialTransfer` transfer mint authority..
    pub mint_authority: &'a Pubkey,
    /// Determines if newly configured accounts must be approved by the
    /// `authority` before they may be used by the user.
    pub auto_approve_new_accounts: bool,
    /// New authority to decode any transfer amount in a confidential transfer.
    pub auditor_elgamal_pubkey: Option<&'a ElagamalPubkey>,
}

impl<'a> UpdateMint<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..33]: mint_authority (32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 33];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[27]);
        // Set mint_authority as Pubkey at offset [1..33]
        write_bytes(&mut instruction_data[1..33], self.mint_authority);

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 33) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
