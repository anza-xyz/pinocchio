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

/// Updates the update authority of a `TokenGroup`.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The group account.
///   1. `[signer]` The current group update authority.
pub struct UpdateGroupAuthority<'a, 'b> {
    /// The group account.
    pub group: &'a AccountView,

    /// The current group update authority.
    pub update_authority: &'a AccountView,

    /// The new update authority, or `None` to make the group immutable.
    pub new_authority: Option<&'b Address>,

    /// The token program.
    pub token_program: &'b Address,
}

impl UpdateGroupAuthority<'_, '_> {
    /// Hash of `spl_token_group_interface:update_authority`.
    pub const DISCRIMINATOR: [u8; 8] = [161, 105, 88, 1, 237, 221, 216, 203];

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
                    InstructionAccount::writable(self.group.address()),
                    InstructionAccount::readonly_signer(self.update_authority.address()),
                ],
                // SAFETY: instruction data is initialized.
                data: unsafe {
                    from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len())
                },
            },
            &[self.group, self.update_authority],
            signers,
        )
    }
}
