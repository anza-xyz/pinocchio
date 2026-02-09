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

/// Permissionless instruction to transfer all withheld tokens to the mint.
///
/// Succeeds for frozen accounts.
///
/// Accounts provided should include the `TransferFeeAmount` extension. If
/// not, the account is skipped.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint.
///   1. `..1+N` `[writable]` The source accounts to harvest from.
pub struct HarvestWithheldTokensToMint<'a, 'b, 'c> {
    /// The token mint.
    pub mint: &'a AccountView,

    /// The source accounts to harvest from.
    pub sources: &'c [&'a AccountView],

    /// The token program.
    pub token_program: &'b Address,
}

impl HarvestWithheldTokensToMint<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 4;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let expected_accounts = 1 + self.sources.len();

        if expected_accounts > MAX_STATIC_CPI_ACCOUNTS {
            return Err(ProgramError::InvalidArgument);
        }

        // Instruction accounts.

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; MAX_STATIC_CPI_ACCOUNTS];

        // SAFETY: The expected number of accounts has been validated to be less than
        // the maximum allocated.
        unsafe {
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.mint.address()));

            for (instruction_account, source) in instruction_accounts
                .get_unchecked_mut(1..)
                .iter_mut()
                .zip(self.sources.iter())
            {
                instruction_account.write(InstructionAccount::writable(source.address()));
            }
        }

        // Accounts.

        let mut accounts = [UNINIT_ACCOUNT_REF; MAX_STATIC_CPI_ACCOUNTS];

        // SAFETY: The expected number of accounts has been validated to be less than
        // the maximum allocated.
        unsafe {
            accounts.get_unchecked_mut(0).write(self.mint);

            for (account, source) in accounts
                .get_unchecked_mut(1..)
                .iter_mut()
                .zip(self.sources.iter())
            {
                account.write(*source);
            }
        }

        invoke_with_bounds::<MAX_STATIC_CPI_ACCOUNTS>(
            &InstructionView {
                program_id: self.token_program,
                // SAFETY: instruction accounts has `expected_accounts` initialized.
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                data: &[
                    ExtensionDiscriminator::TransferFee as u8,
                    Self::DISCRIMINATOR,
                ],
            },
            // SAFETY: accounts has `expected_accounts` initialized.
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
        )
    }
}
