use {
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initialize the non transferable extension for the given mint account.
///
/// Fails if the account has already been initialized, so must be called
/// before `InitializeMint`.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]`  The mint account to initialize.
pub struct InitializeNonTransferableMint<'a, 'b> {
    /// The mint account to initialize.
    pub mint: &'a AccountView,

    /// The token program.
    pub token_program: &'b Address,
}

impl InitializeNonTransferableMint<'_, '_> {
    pub const DISCRIMINATOR: u8 = 32;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        invoke(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[InstructionAccount::writable(self.mint.address())],
                data: &[Self::DISCRIMINATOR],
            },
            &[self.mint],
        )
    }
}
