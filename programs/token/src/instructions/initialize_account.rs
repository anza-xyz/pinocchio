use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

use super::TokenProgramVariant;

/// Initialize a new Token Account.
///
/// ### Accounts:
///   0. `[WRITE]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
///   2. `[]` The new account's owner/multisignature.
///   3. `[]` Rent sysvar
pub struct InitializeAccount<'a> {
    /// New Account.
    pub account: &'a AccountInfo,
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Owner of the new Account.
    pub owner: &'a AccountInfo,
    /// Rent Sysvar Account
    pub rent_sysvar: &'a AccountInfo,
}

impl InitializeAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self, token_program: TokenProgramVariant) -> ProgramResult {
        self.invoke_signed(&[], token_program)
    }

    pub fn invoke_signed(
        &self,
        signers: &[Signer],
        token_program: TokenProgramVariant,
    ) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly(self.owner.key()),
            AccountMeta::readonly(self.rent_sysvar.key()),
        ];

        let instruction = Instruction {
            program_id: &token_program.into(),
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
