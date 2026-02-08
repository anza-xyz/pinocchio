use {
    crate::{instructions::extensions::ExtensionDiscriminator, write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initialize the pausable extension for the given mint account
///
/// Fails if the account has already been initialized, so must be called
/// before `InitializeMint`.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]`  The mint account to initialize.
pub struct Initialize<'a, 'b> {
    /// The mint account to initialize.
    pub mint: &'a AccountView,

    /// The address for the account that can pause the mint.
    pub authority: &'b Address,

    /// The token program.
    pub token_program: &'b Address,
}

impl Initialize<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 34];

        // discriminators
        write_bytes(
            &mut instruction_data[..2],
            &[ExtensionDiscriminator::Pausable as u8, Self::DISCRIMINATOR],
        );
        // authority
        write_bytes(&mut instruction_data[2..34], self.authority.as_ref());

        // Instruction.

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &[InstructionAccount::writable(self.mint.address())],
            data: unsafe {
                from_raw_parts(
                    instruction_data.as_ptr() as *const _,
                    instruction_data.len(),
                )
            },
        };

        invoke(&instruction, &[self.mint])
    }
}
