use {
    core::{mem::MaybeUninit, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_unchecked, CpiAccount},
        error::ProgramError,
        instruction::{InstructionAccount, InstructionView},
        AccountView, ProgramResult,
    },
};

/// One-time idempotent upgrade of legacy nonce versions in order to bump
/// them out of chain blockhash domain.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
pub struct UpgradeNonceAccount<'account> {
    /// Nonce account.
    pub account: &'account AccountView,
}

impl UpgradeNonceAccount<'_> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u32 = 12;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Instruction accounts.
        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 1];
        instruction_accounts[0].write(InstructionAccount::writable(self.account.address()));

        let instruction = InstructionView {
            program_id: &crate::ID,
            // SAFETY: `instruction_accounts` was initialized.
            accounts: unsafe { from_raw_parts(instruction_accounts.as_ptr() as _, 1) },
            data: &Self::DISCRIMINATOR.to_le_bytes(),
        };

        if self.account.is_borrowed() {
            return Err(ProgramError::AccountBorrowFailed);
        }

        let mut accounts = [const { MaybeUninit::<CpiAccount>::uninit() }; 1];
        CpiAccount::init_from_account_view(self.account, &mut accounts[0]);

        // SAFETY: `accounts` was initialized and not borrowed.
        unsafe {
            invoke_signed_unchecked(&instruction, from_raw_parts(accounts.as_ptr() as _, 1), &[])
        };

        Ok(())
    }
}
