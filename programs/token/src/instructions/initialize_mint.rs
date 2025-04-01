use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    Address,
    cpi::invoke,
    instruction::{AccountMeta, Instruction},
    ProgramResult,
};

use crate::{write_bytes, UNINIT_BYTE};

/// Initialize a new mint.
///
/// ### Accounts:
///   0. `[WRITABLE]` Mint account
///   1. `[]` Rent sysvar
pub struct InitializeMint<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Rent sysvar Account.
    pub rent_sysvar: &'a AccountInfo,
    /// Decimals.
    pub decimals: u8,
    /// Mint Authority.
    pub mint_authority: &'a Address,
    /// Freeze Authority.
    pub freeze_authority: Option<&'a Address>,
}

impl InitializeMint<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly(self.rent_sysvar.key()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: decimals (1 byte, u8)
        // -  [2..34]: mint_authority (32 bytes, Address)
        // -  [34]: freeze_authority presence flag (1 byte, u8)
        // -  [35..67]: freeze_authority (optional, 32 bytes, Address)
        let mut instruction_data = [UNINIT_BYTE; 67];
        let mut length = instruction_data.len();

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[0]);
        // Set decimals as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[self.decimals]);
        // Set mint_authority as Address at offset [2..34]
        write_bytes(&mut instruction_data[2..34], self.mint_authority.as_array());

        if let Some(freeze_auth) = self.freeze_authority {
            // Set Option = `true` & freeze_authority at offset [34..67]
            write_bytes(&mut instruction_data[34..35], &[1]);
            write_bytes(&mut instruction_data[35..], freeze_auth.as_array());
        } else {
            // Set Option = `false`
            write_bytes(&mut instruction_data[34..35], &[0]);
            // Adjust length if no freeze authority
            length = 35;
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, length) },
        };

        invoke(&instruction, &[self.mint, self.rent_sysvar])
    }
}
