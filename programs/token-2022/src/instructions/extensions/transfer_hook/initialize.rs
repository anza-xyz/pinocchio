use {
    crate::{instructions::extensions::ExtensionDiscriminator, write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initialize a new mint with a transfer hook program.
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
pub struct InitializeTransferHook<'a, 'b> {
    /// The mint to initialize.
    pub mint: &'a AccountView,

    /// The address for the account that can update the program id.
    pub authority: Option<&'b Address>,

    /// The program id that performs logic during transfers.
    pub program_id: Option<&'b Address>,

    /// The token program.
    pub token_program: &'b Address,
}

impl InitializeTransferHook<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 66];

        instruction_data[0].write(ExtensionDiscriminator::TransferHook as u8);

        instruction_data[1].write(Self::DISCRIMINATOR);

        write_bytes(
            &mut instruction_data[2..34],
            if let Some(authority) = self.authority {
                authority.as_ref()
            } else {
                &[0; 32]
            },
        );

        write_bytes(
            &mut instruction_data[34..66],
            if let Some(program_id) = self.program_id {
                program_id.as_ref()
            } else {
                &[0; 32]
            },
        );

        invoke(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[InstructionAccount::writable(self.mint.address())],
                // SAFETY: instruction data is initialized.
                data: unsafe {
                    from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len())
                },
            },
            &[self.mint],
        )
    }
}
