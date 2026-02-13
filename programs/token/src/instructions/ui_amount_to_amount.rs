use {
    crate::{write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::{ProgramError, ProgramResult},
};

/// Convert a `UiAmount` of tokens to a little-endian `u64` raw Amount,
/// using the given mint.
///
/// In particular, conversions will not always work in reverse. For example,
/// if you pass amount `A` to `UiAmountToAmount` and receive `B`, and pass
/// the result `B` to `AmountToUiAmount`, you will not always get back `A`.
///
/// Accounts expected by this instruction:
///
///   0. `[]` The mint to calculate for.
pub struct UiAmountToAmount<'a, 'b, const LENGTH: usize> {
    /// The mint to calculate for.
    pub mint: &'a AccountView,

    /// The amount of tokens to convert.
    pub amount: &'b str,
}

impl<const LENGTH: usize> UiAmountToAmount<'_, '_, LENGTH> {
    pub const DISCRIMINATOR: u8 = 24;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Reserve 1 byte for the instruction discriminator, and the rest for
        // the amount string.
        let expected_data = 1 + self.amount.len();

        if expected_data > LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; LENGTH];

        instruction_data[0].write(Self::DISCRIMINATOR);

        write_bytes(&mut instruction_data[1..], self.amount.as_bytes());

        invoke(
            &InstructionView {
                program_id: &crate::ID,
                accounts: &[InstructionAccount::readonly(self.mint.address())],
                // SAFETY: `instruction_data` was initialized.
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, expected_data) },
            },
            &[self.mint],
        )
    }
}
