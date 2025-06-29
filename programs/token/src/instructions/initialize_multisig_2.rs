use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    program_error::ProgramError,
    ProgramResult,
};

extern crate alloc;

use alloc::vec::Vec;

use crate::{write_bytes, UNINIT_BYTE};

/// Initialize a new Multisig.
///
/// ### Accounts:
///   0. `[writable]` The multisig account to initialize.
///   1. `[]` Rent sysvar
///   2. ..`2+N`. `[]` The signer accounts, must equal to N where `1 <= N <=
///      11`.
pub struct InitializeMultisig2<'a> {
    /// Multisig Account.
    pub multisig: &'a AccountInfo,
    /// Signer Accounts
    pub multisig_signers: Vec<&'a AccountInfo>,
    /// The number of signers (M) required to validate this multisignature
    /// account.
    pub m: u8,
}

impl InitializeMultisig2<'_> {
    #[inline(always)]
    pub fn invoke<const ACCOUNTS: usize>(&self) -> ProgramResult {
        self.invoke_signed::<ACCOUNTS>(&[])
    }

    pub fn invoke_signed<const ACCOUNTS: usize>(&self, signers: &[Signer]) -> ProgramResult {
        if ACCOUNTS != self.multisig_signers.len() + 1 {
            return Err(ProgramError::InvalidArgument);
        }

        // Account metadata
        let mut account_metas = Vec::with_capacity(1 + self.multisig_signers.len());
        account_metas.push(AccountMeta::writable(self.multisig.key()));

        account_metas.extend(
            self.multisig_signers
                .iter()
                .map(|a| AccountMeta::readonly(a.key())),
        );

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: m (1 byte, u8)
        let mut instruction_data = [UNINIT_BYTE; 2];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[2]);
        // Set number of signers (m) at offset 1
        write_bytes(&mut instruction_data[1..2], &[self.m]);

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: account_metas.as_slice(),
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 2) },
        };

        let mut account_infos = Vec::with_capacity(1 + self.multisig_signers.len());

        account_infos.push(self.multisig);

        account_infos.extend_from_slice(self.multisig_signers.as_slice());

        let account_infos: [&AccountInfo; ACCOUNTS] = account_infos
            .try_into()
            .map_err(|_| ProgramError::InvalidArgument)?;

        invoke_signed(&instruction, &account_infos, signers)
    }
}
