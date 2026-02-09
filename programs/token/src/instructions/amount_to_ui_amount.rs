use {
    crate::{write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Convert an Amount of tokens to a `UiAmount` string, using the given
/// mint.
///
/// Fails on an invalid mint.
///
/// Return data can be fetched using `sol_get_return_data` and deserialized
/// with `String::from_utf8`.
///
/// Accounts expected by this instruction:
///
///   0. `[]` The mint to calculate for.
pub struct AmountToUiAmount<'a> {
    /// The mint to calculate for.
    pub mint: &'a AccountView,

    /// The amount of tokens to convert.
    pub amount: u64,
}

impl AmountToUiAmount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 9];

        // discriminator
        instruction_data[0].write(23);
        // amount
        write_bytes(&mut instruction_data[1..9], &self.amount.to_le_bytes());

        invoke(
            &InstructionView {
                program_id: &crate::ID,
                accounts: &[InstructionAccount::readonly(self.mint.address())],
                // SAFETY: `instruction_data` was initialized.
                data: unsafe {
                    from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len())
                },
            },
            &[self.mint],
        )
    }
}
