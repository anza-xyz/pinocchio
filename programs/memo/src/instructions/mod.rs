use core::mem::MaybeUninit;

use solana_account_view::AccountView;
use solana_instruction_view::{
    cpi::{slice_invoke_signed, Signer, MAX_CPI_ACCOUNTS},
    AccountRole, InstructionView,
};
use solana_program_error::{ProgramError, ProgramResult};

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
        const UNINIT_META: MaybeUninit<AccountRole> = MaybeUninit::<AccountRole>::uninit();

        // We don't know num_accounts at compile time, so we use MAX_CPI_ACCOUNTS
        let mut account_metas = [UNINIT_META; MAX_CPI_ACCOUNTS];

        let num_accounts = self.signers.len();
        if num_accounts > MAX_CPI_ACCOUNTS {
            return Err(ProgramError::InvalidArgument);
        }

        for i in 0..num_accounts {
            unsafe {
                // SAFETY: num_accounts is less than MAX_CPI_ACCOUNTS
                // SAFETY: i is less than len(self.signers)
                account_metas
                    .get_unchecked_mut(i)
                    .write(AccountRole::readonly_signer(
                        self.signers.get_unchecked(i).address(),
                    ));
            }
        }

        // SAFETY: len(account_metas) <= MAX_CPI_ACCOUNTS
        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: unsafe {
                core::slice::from_raw_parts(account_metas.as_ptr() as _, num_accounts)
            },
            data: self.memo.as_bytes(),
        };

        slice_invoke_signed(&instruction, self.signers, signers_seeds)
    }
}
