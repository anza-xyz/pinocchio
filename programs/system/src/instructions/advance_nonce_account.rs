use {
    core::{mem::MaybeUninit, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_unchecked, CpiAccount, Signer},
        error::ProgramError,
        instruction::{InstructionAccount, InstructionView},
        AccountView, ProgramResult,
    },
};

/// Consumes a stored nonce, replacing it with a successor.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[]` Recent blockhashes sysvar
///   2. `[SIGNER]` Nonce authority
pub struct AdvanceNonceAccount<'account> {
    /// Nonce account.
    pub account: &'account AccountView,

    /// Recent blockhashes sysvar.
    pub recent_blockhashes_sysvar: &'account AccountView,

    /// Nonce authority.
    pub authority: &'account AccountView,
}

impl AdvanceNonceAccount<'_> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u32 = 4;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts.
        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 3];
        instruction_accounts[0].write(InstructionAccount::writable(self.account.address()));
        instruction_accounts[1].write(InstructionAccount::readonly(
            self.recent_blockhashes_sysvar.address(),
        ));
        instruction_accounts[2].write(InstructionAccount::readonly_signer(
            self.authority.address(),
        ));

        // instruction
        let instruction = InstructionView {
            program_id: &crate::ID,
            // SAFETY: `instruction_accounts` was initialized.
            accounts: unsafe { from_raw_parts(instruction_accounts.as_ptr() as _, 3) },
            data: &Self::DISCRIMINATOR.to_le_bytes(),
        };

        if self.account.is_borrowed() {
            return Err(ProgramError::AccountBorrowFailed);
        }

        let mut accounts = [const { MaybeUninit::<CpiAccount>::uninit() }; 3];
        CpiAccount::init_from_account_view(self.account, &mut accounts[0]);
        CpiAccount::init_from_account_view(self.recent_blockhashes_sysvar, &mut accounts[1]);
        CpiAccount::init_from_account_view(self.authority, &mut accounts[2]);

        // SAFETY: `accounts` was initialized and not borrowed.
        unsafe {
            invoke_signed_unchecked(
                &instruction,
                from_raw_parts(accounts.as_ptr() as _, 3),
                signers,
            )
        };

        Ok(())
    }
}
