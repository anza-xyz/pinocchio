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

/// Updates the maximum number of members of a `TokenGroup`.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The group account.
///   1. `[signer]` The group update authority.
pub struct UpdateGroupMaxSize<'a, 'b> {
    /// The group account.
    pub group: &'a AccountView,

    /// The group update authority.
    pub update_authority: &'a AccountView,

    /// The new maximum number of group members.
    pub max_size: u64,

    /// The token program.
    pub token_program: &'b Address,
}

impl UpdateGroupMaxSize<'_, '_> {
    /// Hash of `spl_token_group_interface:update_group_max_size`.
    pub const DISCRIMINATOR: [u8; 8] = [108, 37, 171, 143, 248, 30, 18, 110];

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 16];

        write_bytes(&mut instruction_data[0..8], &Self::DISCRIMINATOR);

        write_bytes(&mut instruction_data[8..16], &self.max_size.to_le_bytes());

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
