use {
    crate::instructions::ExtensionDiscriminator,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::ProgramResult,
};

/// Approves a token account for confidential transfers.
///
/// Approval is only required when the
/// `ConfidentialTransferMint::approve_new_accounts` field is set in the
/// SPL Token mint. This instruction must be executed after the account
/// owner configures their account for credential transfers with
/// `ConfidentialTransferInstruction::ConfigureAccount`.
///
/// Accounts expected by this instruction:
///
/// 0. `[writable]` The SPL Token account to approve.
/// 1. `[]` The SPL Token mint.
/// 2. `[signer]` Confidential transfer mint authority.
pub struct ApproveAccount<'a> {
    /// Token Account to be approved for confidential transfers
    pub token_account: &'a AccountView,
    /// The Token mint
    pub mint: &'a AccountView,
    /// Confidential Mint Authority to approve new account
    pub authority: &'a AccountView,
    /// The token program
    pub token_program: &'a Address,
}

impl ApproveAccount<'_> {
    const DISCRIMINATOR: u8 = 3;

    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let instruction_accounts = [
            InstructionAccount::writable(self.token_account.address()),
            InstructionAccount::readonly(self.mint.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ];

        let instruction_data = [
            ExtensionDiscriminator::ConfidentialTransfer as u8,
            ApproveAccount::DISCRIMINATOR,
        ];

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &instruction_accounts,
            data: instruction_data.as_slice(),
        };

        invoke_signed(
            &instruction,
            &[self.token_account, self.mint, self.authority],
            signers,
        )
    }
}
