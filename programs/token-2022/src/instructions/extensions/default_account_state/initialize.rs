use {
    crate::{instructions::extensions::ExtensionDiscriminator, state::AccountState},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initialize a new mint with the default state for new Accounts.
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
pub struct Initialize<'a, 'b> {
    /// The mint to initialize.
    pub mint: &'a AccountView,

    /// The default account state in which new token accounts should be
    /// initialized.
    pub state: AccountState,

    /// The token program.
    pub token_program: &'b Address,
}

impl Initialize<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        invoke(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[InstructionAccount::writable(self.mint.address())],
                data: &[
                    ExtensionDiscriminator::DefaultAccountState as u8,
                    Self::DISCRIMINATOR,
                    self.state as u8,
                ],
            },
            &[self.mint],
        )
    }
}
