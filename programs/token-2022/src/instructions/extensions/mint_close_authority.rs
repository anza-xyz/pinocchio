use {
    crate::{instructions::ExtensionDiscriminator, write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initialize the close account authority on a new mint.
///
/// Fails if the mint has already been initialized, so must be called before
/// `InitializeMint`.
///
/// The mint must have exactly enough space allocated for the base mint (82
/// bytes), plus 83 bytes of padding, 1 byte reserved for the account type,
/// then space required for this extension, plus any others.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint to initialize.
pub struct InitializeMintCloseAuthority<'a, 'b> {
    /// The mint to initialize.
    pub mint: &'a AccountView,

    /// Authority that must sign the `CloseAccount` instruction on a mint.
    pub close_authority: Option<&'b Address>,
}

impl InitializeMintCloseAuthority<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 34];
        let mut expected_data = 2;

        // discriminator
        instruction_data[0].write(ExtensionDiscriminator::MintCloseAuthority as u8);
        // close_authority
        if let Some(authority) = self.close_authority {
            instruction_data[1].write(1);
            write_bytes(&mut instruction_data[2..34], authority.as_ref());
            expected_data += 32;
        } else {
            instruction_data[1].write(0);
        }

        invoke(
            &InstructionView {
                program_id: &crate::ID,
                accounts: &[InstructionAccount::writable(self.mint.address())],
                // SAFETY: `instruction_data` was initialized.
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, expected_data) },
            },
            &[self.mint],
        )
    }
}
