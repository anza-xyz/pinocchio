use core::mem::MaybeUninit;

use pinocchio::{
    account::AccountView,
    cpi::{slice_invoke_signed, MAX_CPI_ACCOUNTS},
    error::ProgramError,
    instruction::{AccountMeta, Instruction, Signer},
    ProgramResult,
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
        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();

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
                    .write(AccountMeta::readonly_signer(
                        self.signers.get_unchecked(i).address(),
                    ));
            }
        }

        // SAFETY: len(account_metas) <= MAX_CPI_ACCOUNTS
        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: unsafe {
                core::slice::from_raw_parts(account_metas.as_ptr() as _, num_accounts)
            },
            data: self.memo.as_bytes(),
        };

        slice_invoke_signed(&instruction, self.signers, signers_seeds)
    }
}
