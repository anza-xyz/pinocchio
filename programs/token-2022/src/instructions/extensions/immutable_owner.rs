use solana_account_view::AccountView;
use solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView};
use solana_program_error::ProgramResult;

use crate::instructions::ExtensionDiscriminator;

/// Initialize the Immutable-Owner extension on a token account.
///
/// Expected accounts:
///
/// 0. `[writable]` The token account to initialize with immutable owner.
pub struct InitializeImmutableOwner<'a> {
    /// The token account to initialize with the Immutable-Owner extension.
    pub token_account: &'a AccountView,
}

impl InitializeImmutableOwner<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let accounts = [InstructionAccount::writable(self.token_account.address())];

        let data = &[ExtensionDiscriminator::ImmutableOwner as u8];

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &accounts,
            data,
        };

        invoke(&instruction, &[self.token_account])
    }
}
