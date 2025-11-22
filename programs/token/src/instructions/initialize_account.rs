use pinocchio::{
    account::AccountView,
    cpi::invoke,
    instruction::{AccountMeta, Instruction},
    ProgramResult,
};

/// Initialize a new Token Account.
///
/// ### Accounts:
///   0. `[WRITE]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
///   2. `[]` The new account's owner/multi-signature.
///   3. `[]` Rent sysvar
pub struct InitializeAccount<'a> {
    /// New Account.
    pub account: &'a AccountView,
    /// Mint Account.
    pub mint: &'a AccountView,
    /// Owner of the new Account.
    pub owner: &'a AccountView,
    /// Rent Sysvar Account
    pub rent_sysvar: &'a AccountView,
}

impl InitializeAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.account.address()),
            AccountMeta::readonly(self.mint.address()),
            AccountMeta::readonly(self.owner.address()),
            AccountMeta::readonly(self.rent_sysvar.address()),
        ];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &[1],
        };

        invoke(
            &instruction,
            &[self.account, self.mint, self.owner, self.rent_sysvar],
        )
    }
}
