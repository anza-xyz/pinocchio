use pinocchio::{
    cpi::{invoke_signed, Signer},
    instruction::{InstructionAccount, InstructionView},
    AccountView, Address, ProgramResult,
};

/// Change the entity authorized to execute nonce instructions on the account.
///
/// The [`Address`] parameter identifies the entity to authorize.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[SIGNER]` Nonce authority
pub struct AuthorizeNonceAccount<'a, 'b> {
    /// Nonce account.
    pub account: &'a AccountView,

    /// Nonce authority.
    pub authority: &'a AccountView,

    /// New entity authorized to execute nonce instructions on the account.
    pub new_authority: &'b Address,
}

impl AuthorizeNonceAccount<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts
        let instruction_accounts: [InstructionAccount; 2] = [
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ];

        // instruction data
        // - [0..4 ]: instruction discriminator
        // - [4..12]: lamports
        let mut instruction_data = [0; 36];
        instruction_data[0] = 7;
        instruction_data[4..36].copy_from_slice(self.new_authority.as_array());

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            data: &instruction_data,
        };

        invoke_signed(&instruction, &[self.account, self.authority], signers)
    }
}
