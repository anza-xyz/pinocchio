use {
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initialize the Immutable Owner extension for the given token account
///
/// Fails if the account has already been initialized, so must be called
/// before `InitializeAccount`.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]`  The account to initialize.
pub struct InitializeImmutableOwner<'a, 'b> {
    /// The account to initialize.
    pub account: &'a AccountView,

    /// The token program.
    pub token_program: &'b Address,
}

impl InitializeImmutableOwner<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        invoke(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[InstructionAccount::writable(self.account.address())],
                data: &[22],
            },
            &[self.account],
        )
    }
}
