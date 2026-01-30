use {
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::ProgramResult,
};

/// Transfers from and closes a nested associated token account: an
/// associated token account owned by an associated token account.
///
/// The tokens are moved from the nested associated token account to the
/// wallet's associated token account, and the nested account lamports are
/// moved to the wallet.
///
/// Note: Nested token accounts are an anti-pattern, and almost always
/// created unintentionally, so this instruction should only be used to
/// recover from errors
///
/// ### Accounts:
///   0. `[WRITE]` Nested associated token account, must be owned by `3`
///   1. `[]` Token mint for the nested associated token account
///   2. `[WRITE]`  Wallet's associated token account
///   3. `[]` Owner associated token account address, must be owned by `5`
///   4. `[]` Token mint for the owner associated token account
///   5. `[WRITE, SIGNER]` Wallet address for the owner associated token account
///   6. `[]`  SPL Token program
pub struct RecoverNested<'a> {
    /// Nested associated token account, must be owned by
    /// `owner_associated_token_account`
    pub account: &'a AccountView,
    /// Token mint for the nested associated token account
    pub mint: &'a AccountView,
    /// Wallet's associated token account
    pub destination_account: &'a AccountView,
    /// Owner associated token account address, must be owned by
    /// `wallet_account`
    pub owner_account: &'a AccountView,
    /// Token mint for the owner associated token account
    pub owner_mint: &'a AccountView,
    /// Wallet address for the owner associated token account
    pub wallet: &'a AccountView,
    /// SPL Token program
    pub token_program: &'a AccountView,
}

impl RecoverNested<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts
        let instruction_accounts: [InstructionAccount; 7] = [
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::readonly(self.mint.address()),
            InstructionAccount::writable(self.destination_account.address()),
            InstructionAccount::readonly(self.owner_account.address()),
            InstructionAccount::readonly(self.owner_mint.address()),
            InstructionAccount::writable_signer(self.wallet.address()),
            InstructionAccount::readonly(self.token_program.address()),
        ];

        // Instruction data:
        // - [0]: Instruction discriminator (1 byte, u8) (2 for RecoverNested)

        let instruction_data = [2u8];

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            data: &instruction_data,
        };

        invoke_signed(
            &instruction,
            &[
                self.account,
                self.mint,
                self.destination_account,
                self.owner_account,
                self.owner_mint,
                self.wallet,
                self.token_program,
            ],
            signers,
        )
    }
}
