use solana_account_view::AccountView;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    AccountRole, InstructionView,
};
use solana_program_error::ProgramResult;

/// Creates an associated token account for the given wallet address and
/// token mint, if it doesn't already exist.  Returns an error if the
/// account exists, but with a different owner.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Funding account (must be a system account)
///   1. `[WRITE]` Associated token account address to be created
///   2. `[]` Wallet address for the new associated token account
///   3. `[]` The token mint for the new associated token account
///   4. `[]` System program
///   5. `[]` SPL Token program
pub struct CreateIdempotent<'a> {
    /// Funding account (must be a system account)
    pub funding_account: &'a AccountView,
    /// Associated token account address to be created
    pub account: &'a AccountView,
    /// Wallet address for the new associated token account
    pub wallet: &'a AccountView,
    /// The token mint for the new associated token account
    pub mint: &'a AccountView,
    /// System program
    pub system_program: &'a AccountView,
    /// SPL Token program
    pub token_program: &'a AccountView,
}

impl CreateIdempotent<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountRole; 6] = [
            AccountRole::writable_signer(self.funding_account.address()),
            AccountRole::writable(self.account.address()),
            AccountRole::readonly(self.wallet.address()),
            AccountRole::readonly(self.mint.address()),
            AccountRole::readonly(self.system_program.address()),
            AccountRole::readonly(self.token_program.address()),
        ];

        // Instruction data:
        // - [0]: Instruction discriminator (1 byte, u8) (1 for CreateIdempotent)

        let instruction_data = [1u8];

        let instruction = InstructionView {
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
