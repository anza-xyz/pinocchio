use {
    crate::{instructions::ExtensionDiscriminator, UNINIT_ACCOUNT_REF, UNINIT_INSTRUCTION_ACCOUNT},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_with_bounds, MAX_STATIC_CPI_ACCOUNTS},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Permissionless instruction to transfer all withheld confidential tokens
/// to the mint.
///
/// Succeeds for frozen accounts.
///
/// Accounts provided should include both the `TransferFeeAmount` and
/// `ConfidentialTransferAccount` extension. If not, the account is skipped.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint.
///   1. ..`1+N` `[writable]` The source accounts to harvest from.
pub struct HarvestWithheldTokensToMint<'a, 'b> {
    /// The token mint
    pub mint: &'a AccountView,
    /// The source accounts
    pub source_accounts: &'b [&'a AccountView],
    /// The token program
    pub token_program: &'a Address,
}

impl HarvestWithheldTokensToMint<'_, '_> {
    pub const DISCRIMINATOR: u8 = 3;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let expected_accounts = 1 + self.source_accounts.len();

        if expected_accounts > MAX_STATIC_CPI_ACCOUNTS {
            return Err(ProgramError::InvalidArgument);
        }

        // Instruction Accounts, and Cpi Accounts

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; MAX_STATIC_CPI_ACCOUNTS];
        let mut accounts = [UNINIT_ACCOUNT_REF; MAX_STATIC_CPI_ACCOUNTS];

        // SAFETY: The allocation is validated to the maximum number of accounts
        unsafe {
            // The token mint
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.mint.address()));

            // The source accounts
            for ((instruction_account, account), source_account) in instruction_accounts
                .get_unchecked_mut(1..)
                .iter_mut()
                .zip(accounts.get_unchecked_mut(1..).iter_mut())
                .zip(self.source_accounts.iter())
            {
                instruction_account.write(InstructionAccount::writable(source_account.address()));
                account.write(*source_account);
            }
        };

        // instruction
        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: &[
                ExtensionDiscriminator::ConfidentialTransferFee as u8,
                Self::DISCRIMINATOR,
            ],
        };

        invoke_with_bounds::<MAX_STATIC_CPI_ACCOUNTS>(&instruction, unsafe {
            from_raw_parts(accounts.as_ptr() as _, expected_accounts)
        })
    }
}
