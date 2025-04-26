use alloc::vec::Vec;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};

/// Extend an address lookup table with new addresses. Funding account and
/// system program account references are only required if the lookup table
/// account requires additional lamports to cover the rent-exempt balance
/// after being extended.
///
/// # Account references
///   0. `[WRITE]` Address lookup table account to extend
///   1. `[SIGNER]` Current authority
///   2. `[SIGNER, WRITE, OPTIONAL]` Account that will fund the table
///      reallocation
///   3. `[OPTIONAL]` System program for CPI.
pub struct Extend<'a> {
    /// Address lookup table account to deactivate
    pub lookup_table: &'a AccountInfo,
    /// Current authority
    pub authority: &'a AccountInfo,
    /// Account that will fund the table reallocation
    pub payer: Option<&'a AccountInfo>,
    /// System program for CPI.
    pub system_program: Option<&'a AccountInfo>,
    /// Addresses to extend the table with
    pub new_addresses: Vec<&'a Pubkey>,
}

impl Extend<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data:
        // - [0]: Instruction discriminator (4 bytes, u32) (2 for Extend)

        // LOOKUP_TABLE_MAX_ADDRESSES == u8::MAX + 1.
        let mut instruction_data = [0; 8196]; // TODO: huge for stack, maybe forget about no_std and use vector
        instruction_data[0] = 2;
        for (i, address) in (self).new_addresses.iter().enumerate() {
            let offset = 4 + i * 32;
            instruction_data[offset..offset + 32].copy_from_slice(address.as_ref());
        }
        // account metadata
        match self.payer {
            Some(payer) => {
                let account_metas: [AccountMeta; 4] = [
                    AccountMeta::writable(self.lookup_table.key()),
                    AccountMeta::readonly_signer(self.authority.key()),
                    AccountMeta::writable_signer(payer.key()),
                    AccountMeta::readonly(self.system_program.unwrap().key()),
                ];
                let instruction = Instruction {
                    program_id: &crate::ID,
                    accounts: &account_metas,
                    data: &instruction_data[..4 + self.new_addresses.len() * 32],
                };

                invoke_signed(
                    &instruction,
                    &[
                        self.lookup_table,
                        self.authority,
                        payer,
                        self.system_program.unwrap(),
                    ],
                    signers,
                )
            }
            None => {
                let account_metas: [AccountMeta; 2] = [
                    AccountMeta::writable(self.lookup_table.key()),
                    AccountMeta::readonly_signer(self.authority.key()),
                ];
                let instruction = Instruction {
                    program_id: &crate::ID,
                    accounts: &account_metas,
                    data: &instruction_data[..4 + self.new_addresses.len() * 32],
                };

                invoke_signed(&instruction, &[self.lookup_table, self.authority], signers)
            }
        }
    }
}
