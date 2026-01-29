use {
    crate::instructions::extensions::ExtensionDiscriminator,
    core::slice,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke_with_bounds, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

pub struct Initialize<'a, 'b> {
    /// Mint account to initialize
    pub mint_account: &'a AccountView,
    /// Default Account::state in which new Accounts should be initialized
    pub state: u8,
    /// Token Program
    pub token_program: &'b Address,
}

impl Initialize<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let instruction_accounts = [InstructionAccount::writable(self.mint_account.address())];

        let data = &[
            ExtensionDiscriminator::DefaultAccountState as u8,
            Self::DISCRIMINATOR,
            self.state,
        ];

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe { slice::from_raw_parts(instruction_accounts.as_ptr() as _, 1) },
            data,
        };

        let account_views = [self.mint_account];

        invoke_with_bounds::<1>(&instruction, unsafe {
            slice::from_raw_parts(account_views.as_ptr() as _, 1)
        })
    }
}
