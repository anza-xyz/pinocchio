use {
    crate::{write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Removes a key-value pair from the additional metadata of a
/// `TokenMetadata` entry.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The metadata account.
///   1. `[signer]` The update authority.
pub struct RemoveKey<'a, 'b> {
    /// The metadata account.
    pub metadata: &'a AccountView,

    /// The update authority.
    pub update_authority: &'a AccountView,

    /// If `true`, the instruction does not fail when the key does not
    /// exist.
    pub idempotent: bool,

    /// The key to remove.
    pub key: &'b str,

    /// The token program.
    pub token_program: &'b Address,
}

impl RemoveKey<'_, '_> {
    /// Hash of `spl_token_metadata_interface:remove_key_ix`.
    pub const DISCRIMINATOR: [u8; 8] = [234, 18, 32, 56, 89, 141, 37, 181];

    /// Maximum instruction data length.
    pub const MAX_DATA_LEN: usize = 1024;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data.

        let expected_data = 8 + 1 + 4 + self.key.len();

        if expected_data > Self::MAX_DATA_LEN {
            return Err(ProgramError::InvalidArgument);
        }

        let mut instruction_data = [UNINIT_BYTE; Self::MAX_DATA_LEN];

        write_bytes(&mut instruction_data[0..8], &Self::DISCRIMINATOR);

        instruction_data[8].write(self.idempotent as u8);

        write_bytes(
            &mut instruction_data[9..13],
            &(self.key.len() as u32).to_le_bytes(),
        );

        write_bytes(
            &mut instruction_data[13..expected_data],
            self.key.as_bytes(),
        );

        invoke_signed(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[
                    InstructionAccount::writable(self.metadata.address()),
                    InstructionAccount::readonly_signer(self.update_authority.address()),
                ],
                // SAFETY: instruction data is initialized up to `expected_data`.
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, expected_data) },
            },
            &[self.metadata, self.update_authority],
            signers,
        )
    }
}
