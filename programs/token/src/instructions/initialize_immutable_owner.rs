use {
    solana_account_view::AccountView,
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
pub struct InitializeImmutableOwner<'a> {
    /// The account to initialize.
    pub account: &'a AccountView,
}

impl InitializeImmutableOwner<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        invoke(
            &InstructionView {
                program_id: &crate::ID,
                accounts: &[InstructionAccount::writable(self.account.address())],
                data: &[22],
            },
            &[self.account],
        )
    }
}
