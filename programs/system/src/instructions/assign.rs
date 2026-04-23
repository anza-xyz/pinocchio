use {
    core::{mem::MaybeUninit, ptr::copy_nonoverlapping, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_unchecked, CpiAccount, Signer},
        error::ProgramError,
        instruction::{InstructionAccount, InstructionView},
        AccountView, Address, ProgramResult,
    },
    solana_address::ADDRESS_BYTES,
};

/// Assign account to a program
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Assigned account address
pub struct Assign<'account, 'address> {
    /// Account to be assigned.
    pub account: &'account AccountView,

    /// Program account to assign as owner.
    pub owner: &'address Address,
}

impl Assign<'_, '_> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u32 = 1;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts.
        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 1];
        instruction_accounts[0].write(InstructionAccount::writable_signer(self.account.address()));

        // instruction data
        // - [0..4 ]: instruction discriminator
        // - [4..36]: owner address
        let mut instruction_data = [const { MaybeUninit::<u8>::uninit() }; 36];
        // SAFETY: All writes are within bounds of the allocated data.
        unsafe {
            let dst = instruction_data.as_mut_ptr() as *mut u8;

            copy_nonoverlapping(
                Self::DISCRIMINATOR.to_le_bytes().as_ptr(),
                dst,
                size_of::<u32>(),
            );

            copy_nonoverlapping(self.owner.as_ref().as_ptr(), dst.add(4), ADDRESS_BYTES);
        }

        let instruction = InstructionView {
            program_id: &crate::ID,
            // SAFETY: `instruction_accounts` was initialized.
            accounts: unsafe { from_raw_parts(instruction_accounts.as_ptr() as _, 1) },
            // SAFETY: `instruction_data` was initialized.
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 36) },
        };

        if self.account.is_borrowed() {
            return Err(ProgramError::AccountBorrowFailed);
        }

        let mut accounts = [const { MaybeUninit::<CpiAccount>::uninit() }; 1];
        CpiAccount::init_from_account_view(self.account, &mut accounts[0]);

        // SAFETY: `accounts` was initialized and not borrowed.
        unsafe {
            invoke_signed_unchecked(
                &instruction,
                from_raw_parts(accounts.as_ptr() as _, 1),
                signers,
            )
        };

        Ok(())
    }
}
