use {
    pinocchio::{
        account_info::AccountInfo,
        instruction::{AccountMeta, Instruction, Signer},
        ProgramResult,
    },
    pinocchio_cpi::invoke_signed,
};

/// Creates an associated token account for the given wallet address and token mint.
/// Returns an error if the account exists.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Funding account (must be a system account)
///   1. `[WRITE]` Associated token account address to be created
///   2. `[]` Wallet address for the new associated token account
///   3. `[]` The token mint for the new associated token account
///   4. `[]` System program
///   5. `[]` SPL Token program
pub struct Create<'a> {
    /// Funding account (must be a system account)
    pub funding_account: &'a AccountInfo,
    /// Associated token account address to be created
    pub account: &'a AccountInfo,
    /// Wallet address for the new associated token account
    pub wallet: &'a AccountInfo,
    /// The token mint for the new associated token account
    pub mint: &'a AccountInfo,
    /// System program
    pub system_program: &'a AccountInfo,
    /// SPL Token program
    pub token_program: &'a AccountInfo,
}

impl Create<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 6] = [
            AccountMeta::writable_signer(self.funding_account.key()),
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly(self.wallet.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly(self.system_program.key()),
            AccountMeta::readonly(self.token_program.key()),
        ];

        // Instruction data:
        // - [0]: Instruction discriminator (1 byte, u8) (0 for Create)

        let instruction_data = [0u8];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(
            &instruction,
            &[
                self.funding_account,
                self.account,
                self.wallet,
                self.mint,
                self.system_program,
                self.token_program,
            ],
            signers,
        )
    }
}
