use {
    core::{mem::MaybeUninit, ptr::copy_nonoverlapping, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_unchecked, CpiAccount, Signer},
        error::ProgramError,
        instruction::{InstructionAccount, InstructionView},
        AccountView, ProgramResult,
    },
};

/// Withdraw funds from a nonce account.
///
/// The `u64` parameter is the lamports to withdraw, which must leave the
/// account balance above the rent exempt reserve or at zero.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[WRITE]` Recipient account
///   2. `[]` Recent blockhashes sysvar
///   3. `[]` Rent sysvar
///   4. `[SIGNER]` Nonce authority
pub struct WithdrawNonceAccount<'account> {
    /// Nonce account.
    pub account: &'account AccountView,

    /// Recipient account.
    pub recipient: &'account AccountView,

    /// Recent blockhashes sysvar.
    pub recent_blockhashes_sysvar: &'account AccountView,

    /// Rent sysvar.
    pub rent_sysvar: &'account AccountView,

    /// Nonce authority.
    pub authority: &'account AccountView,

    /// Lamports to withdraw.
    ///
    /// The account balance must be left above the rent exempt reserve
    /// or at zero.
    pub lamports: u64,
}

impl WithdrawNonceAccount<'_> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u32 = 5;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts.
        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 5];
        instruction_accounts[0].write(InstructionAccount::writable(self.account.address()));
        instruction_accounts[1].write(InstructionAccount::writable(self.recipient.address()));
        instruction_accounts[2].write(InstructionAccount::readonly(
            self.recent_blockhashes_sysvar.address(),
        ));
        instruction_accounts[3].write(InstructionAccount::readonly(self.rent_sysvar.address()));
        instruction_accounts[4].write(InstructionAccount::readonly_signer(
            self.authority.address(),
        ));

        // instruction data
        // - [0..4 ]: instruction discriminator
        // - [4..12]: lamports
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
            accounts: unsafe { from_raw_parts(instruction_accounts.as_ptr() as _, 5) },
            // SAFETY: `instruction_data` was initialized.
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 12) },
        };

        if self.account.is_borrowed() | self.recipient.is_borrowed() {
            return Err(ProgramError::AccountBorrowFailed);
        }

        let mut accounts = [const { MaybeUninit::<CpiAccount>::uninit() }; 5];
        CpiAccount::init_from_account_view(self.account, &mut accounts[0]);
        CpiAccount::init_from_account_view(self.recipient, &mut accounts[1]);
        CpiAccount::init_from_account_view(self.recent_blockhashes_sysvar, &mut accounts[2]);
        CpiAccount::init_from_account_view(self.rent_sysvar, &mut accounts[3]);
        CpiAccount::init_from_account_view(self.authority, &mut accounts[4]);

        // SAFETY: `accounts` was initialized and not borrowed.
        unsafe {
            invoke_signed_unchecked(
                &instruction,
                from_raw_parts(accounts.as_ptr() as _, 5),
                signers,
            )
        };

        Ok(())
    }
}
