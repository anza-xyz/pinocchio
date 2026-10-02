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

/// Initializes a new `TokenGroup` in the provided account.
///
/// Token-2022 only supports storing the group in the mint itself, so the
/// group account must be the mint. The mint must have the `GroupPointer`
/// extension pointing to itself and enough lamports to cover the rent of
/// the additional space.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The group account.
///   1. `[]` The mint.
///   2. `[signer]` The mint authority.
pub struct InitializeGroup<'a, 'b> {
    /// The group account.
    pub group: &'a AccountView,

    /// The mint.
    pub mint: &'a AccountView,

    /// The mint authority.
    pub mint_authority: &'a AccountView,

    /// The authority that can sign to update the group.
    pub update_authority: Option<&'b Address>,

    /// The maximum number of group members.
    pub max_size: u64,

    /// The token program.
    pub token_program: &'b Address,
}

impl InitializeGroup<'_, '_> {
    /// Hash of `spl_token_group_interface:initialize_token_group`.
    pub const DISCRIMINATOR: [u8; 8] = [121, 113, 108, 39, 54, 51, 0, 4];

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 48];

        write_bytes(&mut instruction_data[0..8], &Self::DISCRIMINATOR);

        write_bytes(
            &mut instruction_data[8..40],
            if let Some(update_authority) = self.update_authority {
                update_authority.as_ref()
            } else {
                &[0u8; 32]
            },
        );

        write_bytes(&mut instruction_data[40..48], &self.max_size.to_le_bytes());

        invoke_signed(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[
                    InstructionAccount::writable(self.group.address()),
                    InstructionAccount::readonly(self.mint.address()),
                    InstructionAccount::readonly_signer(self.mint_authority.address()),
                ],
                // SAFETY: instruction data is initialized.
                data: unsafe {
                    from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len())
                },
            },
            &[self.group, self.mint, self.mint_authority],
            signers,
        )
    }
}
