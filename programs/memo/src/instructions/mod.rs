use {
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer, MAX_STATIC_CPI_ACCOUNTS},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Memo instruction.
///
/// ### Accounts:
///   0. `..+N` `[SIGNER]` N signing accounts
pub struct Memo<'a, 'b, 'c> {
    /// Signing accounts
    pub signers: &'b [&'a AccountView],
    /// Memo
    pub memo: &'c str,
}

impl Memo<'_, '_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers_seeds: &[Signer]) -> ProgramResult {
        const UNINIT_INSTRUCTION_ACCOUNT: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();

        // We don't know num_accounts at compile time, so we use
        // `MAX_STATIC_CPI_ACCOUNTS`.
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; MAX_STATIC_CPI_ACCOUNTS];

        let num_accounts = self.signers.len();
        if num_accounts > MAX_STATIC_CPI_ACCOUNTS {
            return Err(ProgramError::InvalidArgument);
        }

        for i in 0..num_accounts {
            unsafe {
                // SAFETY: `num_accounts` is less than MAX_STATIC_CPI_ACCOUNTS.
                instruction_accounts.get_unchecked_mut(i).write(
                    InstructionAccount::readonly_signer(self.signers.get_unchecked(i).address()),
                );
            }
        }

        // SAFETY: len(instruction_accounts) <= MAX_CPI_ACCOUNTS
        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: unsafe {
                core::slice::from_raw_parts(instruction_accounts.as_ptr() as _, num_accounts)
            },
            data: self.memo.as_bytes(),
        };

        invoke_signed_with_bounds::<MAX_STATIC_CPI_ACCOUNTS>(
            &instruction,
            self.signers,
            signers_seeds,
        )
    }
}
