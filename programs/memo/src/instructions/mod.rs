use core::{mem::MaybeUninit, slice::from_raw_parts};

use pinocchio::{
    account_info::{AccountInfo, BorrowState},
    cpi::{invoke_signed_unchecked, MAX_CPI_ACCOUNTS},
    instruction::{Account, AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    ProgramResult,
};

/// Memo instruction.
///
/// ### Accounts:
///   0. `..+N` `[SIGNER]` N signing accounts
pub struct Memo<'a, 'b, 'c> {
    /// Signing accounts
    pub signers: &'b [&'a AccountInfo],
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
        if self.signers.len() > MAX_CPI_ACCOUNTS {
            return Err(ProgramError::InvalidArgument);
        }

        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        // We don't know num_accounts at compile time, so we use MAX_CPI_ACCOUNTS.
        let mut account_metas = [UNINIT_META; MAX_CPI_ACCOUNTS];

        account_metas
            .iter_mut()
            .zip(self.signers.iter())
            .try_for_each(|(meta, account)| {
                // Signers are always read-only, so we need to make sure
                // that their account is not borrowed mutably.
                if account.is_borrowed(BorrowState::MutablyBorrowed) {
                    return Err(ProgramError::AccountBorrowFailed);
                }

                meta.write(AccountMeta::readonly_signer(account.key()));

                Ok(())
            })?;

        let instruction = Instruction {
            program_id: &crate::ID,
            // SAFETY: We only process up to `signers.len()` accounts.
            accounts: unsafe { from_raw_parts(account_metas.as_ptr() as _, self.signers.len()) },
            data: self.memo.as_bytes(),
        };

        self.memo(&instruction, signers_seeds);

        Ok(())
    }

    /// Invokes the memo instruction.
    #[inline(never)]
    fn memo(&self, instruction: &Instruction, signer_seeds: &[Signer]) {
        const UNINIT: MaybeUninit<Account> = MaybeUninit::<Account>::uninit();
        let mut accounts = [UNINIT; MAX_CPI_ACCOUNTS];

        accounts
            .iter_mut()
            .zip(self.signers.iter())
            .for_each(|(account, account_info)| {
                account.write(Account::from(*account_info));
            });

        // SAFETY: At this point it is guaranteed that signers are borrowable
        // according to their mutability on the instruction.
        unsafe {
            invoke_signed_unchecked(
                instruction,
                from_raw_parts(accounts.as_ptr() as _, self.signers.len()),
                signer_seeds,
            );
        }
    }
}
