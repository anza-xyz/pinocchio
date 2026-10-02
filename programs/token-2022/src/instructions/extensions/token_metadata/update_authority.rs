use {
    crate::{write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::ProgramResult,
};

/// Updates the update authority of a `TokenMetadata` entry.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The metadata account.
///   1. `[signer]` The current update authority.
pub struct UpdateAuthority<'a, 'b> {
    /// The metadata account.
    pub metadata: &'a AccountView,

    /// The current update authority.
    pub update_authority: &'a AccountView,

    /// The new update authority, or `None` to make the metadata immutable.
    pub new_authority: Option<&'b Address>,

    /// The token program.
    pub token_program: &'b Address,
}

impl UpdateAuthority<'_, '_> {
    /// Hash of `spl_token_metadata_interface:update_the_authority`.
    pub const DISCRIMINATOR: [u8; 8] = [215, 228, 166, 228, 84, 100, 86, 123];

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 40];

        write_bytes(&mut instruction_data[0..8], &Self::DISCRIMINATOR);

        write_bytes(
            &mut instruction_data[8..40],
            if let Some(new_authority) = self.new_authority {
                new_authority.as_ref()
            } else {
                &[0u8; 32]
            },
        );

        invoke_signed(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[
                    InstructionAccount::writable(self.metadata.address()),
                    InstructionAccount::readonly_signer(self.update_authority.address()),
                ],
                // SAFETY: instruction data is initialized.
                data: unsafe {
                    from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len())
                },
            },
            &[self.metadata, self.update_authority],
            signers,
        )
    }
}
