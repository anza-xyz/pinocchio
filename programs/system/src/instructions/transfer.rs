use {
    core::{mem::MaybeUninit, ptr::copy_nonoverlapping, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_unchecked, CpiAccount, Signer},
        error::ProgramError,
        instruction::{InstructionAccount, InstructionView},
        AccountView, ProgramResult,
    },
};

/// Transfer lamports.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Funding account
///   1. `[WRITE]` Recipient account
pub struct Transfer<'account> {
    /// Funding account.
    pub from: &'account AccountView,

    /// Recipient account.
    pub to: &'account AccountView,

    /// Amount of lamports to transfer.
    pub lamports: u64,
}

impl Transfer<'_> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u32 = 2;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts.
        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 2];
        instruction_accounts[0].write(InstructionAccount::writable_signer(self.from.address()));
        instruction_accounts[1].write(InstructionAccount::writable(self.to.address()));

        // instruction data
        // - [0..4 ]: instruction discriminator
        // - [4..12]: lamports amount
        let mut instruction_data = [const { MaybeUninit::<u8>::uninit() }; 12];
        // SAFETY: All writes are within bounds of the allocated data.
        unsafe {
            let dst = instruction_data.as_mut_ptr() as *mut u8;

            copy_nonoverlapping(
                Self::DISCRIMINATOR.to_le_bytes().as_ptr(),
                dst,
                size_of::<u32>(),
            );

            copy_nonoverlapping(
                self.lamports.to_le_bytes().as_ptr(),
                dst.add(4),
                size_of::<u64>(),
            );
        }

        let instruction = InstructionView {
            program_id: &crate::ID,
            // SAFETY: `instruction_accounts` was initialized.
            accounts: unsafe { from_raw_parts(instruction_accounts.as_ptr() as _, 2) },
            // SAFETY: `instruction_data` was initialized.
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 12) },
        };

        if self.from.is_borrowed() | self.to.is_borrowed() {
            return Err(ProgramError::AccountBorrowFailed);
        }

        let mut accounts = [const { MaybeUninit::<CpiAccount>::uninit() }; 2];
        CpiAccount::init_from_account_view(self.from, &mut accounts[0]);
        CpiAccount::init_from_account_view(self.to, &mut accounts[1]);

        // SAFETY: `accounts` was initialized and not borrowed.
        unsafe {
            invoke_signed_unchecked(
                &instruction,
                from_raw_parts(accounts.as_ptr() as _, 2),
                signers,
            )
        };

        Ok(())
    }
}
