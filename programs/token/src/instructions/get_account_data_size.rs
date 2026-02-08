use {
    solana_account_view::AccountView,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
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
pub struct GetAccountDataSize<'a> {
    /// The mint to calculate for.
    pub mint: &'a AccountView,
}

impl GetAccountDataSize<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        invoke(
            &InstructionView {
                program_id: &crate::ID,
                accounts: &[InstructionAccount::readonly(self.mint.address())],
                data: &[21],
            },
            &[self.mint],
        )
    }
}
