use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::{cpi::invoke, AccountRole, InstructionView};
use solana_program_error::ProgramResult;

/// Initialize a new Token Account.
///
/// ### Accounts:
///   0. `[WRITE]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
///   2. `[]` The new account's owner/multi-signature.
///   3. `[]` Rent sysvar
pub struct InitializeAccount<'a, 'b> {
    /// New Account.
    pub account: &'a AccountView,
    /// Mint Account.
    pub mint: &'a AccountView,
    /// Owner of the new Account.
    pub owner: &'a AccountView,
    /// Rent Sysvar Account
    pub rent_sysvar: &'a AccountView,
    /// Token Program
    pub token_program: &'b Address,
}

impl InitializeAccount<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // account metadata
        let account_metas: [AccountRole; 4] = [
            AccountRole::writable(self.account.address()),
            AccountRole::readonly(self.mint.address()),
            AccountRole::readonly(self.owner.address()),
            AccountRole::readonly(self.rent_sysvar.address()),
        ];

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &account_metas,
            data: &[1],
        };

        invoke(
            &instruction,
            &[self.account, self.mint, self.owner, self.rent_sysvar],
        )
    }
}
