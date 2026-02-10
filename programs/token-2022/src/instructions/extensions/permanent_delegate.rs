use {
    crate::{instructions::extensions::ExtensionDiscriminator, write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initialize the permanent delegate on a new mint.
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
pub struct InitializePermanentDelegate<'a, 'b> {
    /// The mint to initialize.
    pub mint: &'a AccountView,

    /// Authority that may sign for `Transfer`s and `Burn`s on any account.
    pub delegate: &'b Address,

    /// The token program.
    pub token_program: &'b Address,
}

impl InitializePermanentDelegate<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 33];

        // discriminator
        instruction_data[0].write(ExtensionDiscriminator::PermanentDelegate as u8);
        // delegate
        write_bytes(&mut instruction_data[1..33], self.delegate.as_ref());

        invoke(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[InstructionAccount::writable(self.mint.address())],
                // SAFETY: `instruction_data` is initialized.
                data: unsafe {
                    from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len())
                },
            },
            &[self.mint],
        )
    }
}
