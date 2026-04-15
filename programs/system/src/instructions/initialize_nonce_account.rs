use {
    core::{mem::MaybeUninit, ptr::copy_nonoverlapping, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_unchecked, CpiAccount},
        error::ProgramError,
        instruction::{InstructionAccount, InstructionView},
        AccountView, Address, ProgramResult,
    },
    solana_address::ADDRESS_BYTES,
};

/// Drive state of Uninitialized nonce account to Initialized, setting the nonce
/// value.
///
/// The [`Address`] parameter specifies the entity authorized to execute nonce
/// instruction on the account
///
/// No signatures are required to execute this instruction, enabling derived
/// nonce account addresses.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[]` Recent blockhashes sysvar
///   2. `[]` Rent sysvar
pub struct InitializeNonceAccount<'account, 'address> {
    /// Nonce account.
    pub account: &'account AccountView,

    /// Recent blockhashes sysvar.
    pub recent_blockhashes_sysvar: &'account AccountView,

    /// Rent sysvar.
    pub rent_sysvar: &'account AccountView,

    /// Indicates the entity authorized to execute nonce
    /// instruction on the account
    pub authority: &'address Address,
}

impl InitializeNonceAccount<'_, '_> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u32 = 6;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Instruction accounts.
        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 3];
        instruction_accounts[0].write(InstructionAccount::writable(self.account.address()));
        instruction_accounts[1].write(InstructionAccount::readonly(
            self.recent_blockhashes_sysvar.address(),
        ));
        instruction_accounts[2].write(InstructionAccount::readonly(self.rent_sysvar.address()));

        // instruction data
        // - [0..4 ]: instruction discriminator
        // - [4..36]: authority address
        let mut instruction_data = [const { MaybeUninit::<u8>::uninit() }; 36];
        // SAFETY: All writes are within bounds of the allocated data.
        unsafe {
            let dst = instruction_data.as_mut_ptr() as *mut u8;

            copy_nonoverlapping(
                Self::DISCRIMINATOR.to_le_bytes().as_ptr(),
                dst,
                size_of::<u32>(),
            );

            copy_nonoverlapping(self.authority.as_ref().as_ptr(), dst.add(4), ADDRESS_BYTES);
        }

        let instruction = InstructionView {
            program_id: &crate::ID,
            // SAFETY: `instruction_accounts` was initialized.
            accounts: unsafe { from_raw_parts(instruction_accounts.as_ptr() as _, 3) },
            // SAFETY: `instruction_data` was initialized.
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 36) },
        };

        if self.account.is_borrowed() {
            return Err(ProgramError::AccountBorrowFailed);
        }

        let mut accounts = [const { MaybeUninit::<CpiAccount>::uninit() }; 3];
        CpiAccount::init_from_account_view(self.account, &mut accounts[0]);
        CpiAccount::init_from_account_view(self.recent_blockhashes_sysvar, &mut accounts[1]);
        CpiAccount::init_from_account_view(self.rent_sysvar, &mut accounts[2]);

        // SAFETY: `accounts` was initialized and not borrowed.
        unsafe {
            invoke_signed_unchecked(&instruction, from_raw_parts(accounts.as_ptr() as _, 3), &[])
        };

        Ok(())
    }
}
