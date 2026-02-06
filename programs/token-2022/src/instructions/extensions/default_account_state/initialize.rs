use {
    crate::instructions::extensions::ExtensionDiscriminator,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initialize the Default-Account-State extension on a mint.
///
/// Expected accounts:
///
/// 0. `[writable]` The mint account to initialize.
pub struct Initialize<'a, 'b> {
    /// The mint account to initialize.
    pub mint_account: &'a AccountView,
    /// The default account state in which new token accounts should be initialized.
    pub state: u8,
    /// Token program (Token-2022).
    pub token_program: &'b Address,
}

impl Initialize<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let accounts = [InstructionAccount::writable(self.mint_account.address())];

        let data = &[
            ExtensionDiscriminator::DefaultAccountState as u8,
            Self::DISCRIMINATOR,
            self.state,
        ];

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &accounts,
            data,
        };

        invoke(&instruction, &[self.mint_account])
    }
}
