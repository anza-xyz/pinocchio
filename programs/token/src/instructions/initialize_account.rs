use solana_account_view::AccountView;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    AccountMeta, InstructionView,
};
use solana_program_error::ProgramResult;

/// Initialize a new Token Account.
///
/// ### Accounts:
///   0. `[WRITE]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
///   2. `[]` The new account's owner/multisignature.
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
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly(self.owner.key()),
            AccountMeta::readonly(self.rent_sysvar.key()),
        ];

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &[1],
        };

        invoke_signed(
            &instruction,
            &[self.account, self.mint, self.owner, self.rent_sysvar],
            signers,
        )
    }
}
