use core::mem::MaybeUninit;

use pinocchio::{
    account_info::AccountInfo,
    cpi::{invoke_signed_unchecked, MAX_CPI_ACCOUNTS},
    instruction::{Account, AccountMeta, Instruction, Signer},
    ProgramResult,
};

/// Memo instruction.
///
/// ### Accounts:
///   0-N. `[SIGNER]` Signers
pub struct Memo<'a> {
    /// Signing accounts
    pub account_infos: &'a [&'a AccountInfo],
    /// Memo
    pub memo: &'a [u8],
}

impl Memo<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        const UNINIT_ACCOUNT: MaybeUninit<Account> = MaybeUninit::<Account>::uninit();

        // We don't know num_accounts at compile time, so we use MAX_CPI_ACCOUNTS
        let mut account_metas = [UNINIT_META; MAX_CPI_ACCOUNTS];
        let mut accounts = [UNINIT_ACCOUNT; MAX_CPI_ACCOUNTS];

        let num_accounts = self.account_infos.len();

        for i in 0..num_accounts {
            unsafe {
                // SAFETY: num_accounts is less than MAX_CPI_ACCOUNTS
                // and accounts are readonly
                accounts
                    .get_unchecked_mut(i)
                    .write(Account::from(self.account_infos[i]));

                // SAFETY: num_accounts is less than MAX_CPI_ACCOUNTS
                account_metas
                    .get_unchecked_mut(i)
                    .write(AccountMeta::readonly_signer(self.account_infos[i].key()));
            }
        }

        // SAFETY: all account_metas are readonly and is_signer
        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: unsafe {
                core::slice::from_raw_parts(account_metas.as_ptr() as _, num_accounts)
            },
            data: self.memo,
        };

        // SAFETY: accounts are validated
        unsafe {
            invoke_signed_unchecked(
                &instruction,
                core::slice::from_raw_parts(accounts.as_ptr() as _, num_accounts),
                signers,
            );
        }

        Ok(())
    }
}
