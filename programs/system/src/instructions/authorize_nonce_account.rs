use {
    pinocchio::{
        account_info::AccountInfo,
        instruction::{AccountMeta, Instruction, Signer},
        pubkey::Pubkey,
        ProgramResult,
    },
    pinocchio_cpi::invoke_signed,
};

/// Change the entity authorized to execute nonce instructions on the account.
///
/// The `Pubkey` parameter identifies the entity to authorize.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[SIGNER]` Nonce authority
pub struct AuthorizeNonceAccount<'a, 'b> {
    /// Nonce account.
    pub account: &'a AccountInfo,

    /// Nonce authority.
    pub authority: &'a AccountInfo,

    /// New entity authorized to execute nonce instructions on the account.
    pub new_authority: &'b Pubkey,
}

impl AuthorizeNonceAccount<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..12]: lamports
        let mut instruction_data = [0; 36];
        instruction_data[0] = 7;
        instruction_data[4..36].copy_from_slice(self.new_authority);

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(&instruction, &[self.account, self.authority], signers)
    }
}
