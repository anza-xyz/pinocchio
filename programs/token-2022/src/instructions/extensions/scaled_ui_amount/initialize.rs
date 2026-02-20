use {
    crate::{instructions::extensions::ExtensionDiscriminator, write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initialize a new mint with scaled UI amounts.
///
/// Fails if the mint has already been initialized, so must be called before
/// `InitializeMint`.
///
/// Fails if the multiplier is less than or equal to 0 or if it's
/// [subnormal](https://en.wikipedia.org/wiki/Subnormal_number).
///
/// The mint must have exactly enough space allocated for the base mint (82
/// bytes), plus 83 bytes of padding, 1 byte reserved for the account type,
/// then space required for this extension, plus any others.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint to initialize.
pub struct Initialize<'a, 'b> {
    /// The mint to initialize.
    pub mint_account: &'a AccountView,

    /// The address for the account that can update the multiplier.
    pub authority: Option<&'b Address>,

    /// The initial multiplier.
    pub multiplier: f64,

    /// The token program.
    pub token_program: &'b Address,
}

impl Initialize<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 42];

        instruction_data[0].write(ExtensionDiscriminator::ScaledUiAmount as u8);

        write_bytes(
            &mut instruction_data[2..34],
            if let Some(authority) = self.authority {
                authority.as_ref()
            } else {
                &[0; 32]
            },
        );

        write_bytes(
            &mut instruction_data[34..42],
            &self.multiplier.to_le_bytes(),
        );

        invoke(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[InstructionAccount::writable(self.mint_account.address())],
                data: unsafe {
                    from_raw_parts(
                        instruction_data.as_ptr() as *const _,
                        instruction_data.len(),
                    )
                },
            },
            &[self.mint_account],
        )
    }
}
