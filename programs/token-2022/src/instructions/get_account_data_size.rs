use {
    crate::{
        instructions::{ExtensionDiscriminator, MAX_EXTENSION_COUNT},
        UNINIT_BYTE,
    },
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::{ProgramError, ProgramResult},
};

/// Gets the required size of an account for the given mint as a
/// little-endian `u64`.
///
/// Return data can be fetched using `sol_get_return_data` and deserializing
/// the return data as a little-endian `u64`.
///
/// Accounts expected by this instruction:
///
///   0. `[]` The mint to calculate for.
pub struct GetAccountDataSize<'a, 'b, 'c> {
    /// The mint to calculate for.
    pub mint: &'a AccountView,

    /// New extension types to include in the reallocated account
    pub extensions: &'c [ExtensionDiscriminator],

    /// The token program.
    pub token_program: &'b Address,
}

impl GetAccountDataSize<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 21;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        if self.extensions.len() > MAX_EXTENSION_COUNT {
            return Err(ProgramError::InvalidArgument);
        }

        let expected_data = 1 + self.extensions.len() * 2;

        // Instruction data.

        // 1 byte (discriminator) + 2 bytes per extension (extension type as `u16`)
        let mut instruction_data = [UNINIT_BYTE; 1 + MAX_EXTENSION_COUNT * 2];

        instruction_data[0].write(Self::DISCRIMINATOR);

        for (i, extension) in self.extensions.iter().enumerate() {
            let offset = 1 + i * 2;
            // SAFETY: `offset` and `offset + 1` are within bounds of `instruction_data`
            // since `extensions.len() <= MAX_EXTENSION_COUNT`.
            //
            // Write the extension type as a little-endian `u16`.
            unsafe {
                instruction_data
                    .get_unchecked_mut(offset)
                    .write(*extension as u8);
                instruction_data.get_unchecked_mut(offset + 1).write(0);
            }
        }

        invoke(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[InstructionAccount::readonly(self.mint.address())],
                // SAFETY: instruction data has `expected_data` initialized bytes.
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, expected_data) },
            },
            &[self.mint],
        )
    }
}
