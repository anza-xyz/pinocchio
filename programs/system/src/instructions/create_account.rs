use {
    core::{mem::MaybeUninit, ptr::copy_nonoverlapping, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_unchecked, CpiAccount, Signer},
        error::ProgramError,
        instruction::{InstructionAccount, InstructionView},
        sysvars::{rent::Rent, Sysvar},
        AccountView, Address, ProgramResult,
    },
    solana_address::ADDRESS_BYTES,
};

/// Create a new account.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Funding account
///   1. `[WRITE, SIGNER]` New account
pub struct CreateAccount<'account, 'address> {
    /// Funding account.
    pub from: &'account AccountView,

    /// New account.
    pub to: &'account AccountView,

    /// Number of lamports to transfer to the new account.
    pub lamports: u64,

    /// Number of bytes of memory to allocate.
    pub space: u64,

    /// Address of program that will own the new account.
    pub owner: &'address Address,
}

impl<'account, 'address> CreateAccount<'account, 'address> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u32 = 0;

    #[deprecated(since = "0.5.0", note = "Use `with_minimum_balance` instead")]
    #[inline(always)]
    pub fn with_minimal_balance(
        from: &'account AccountView,
        to: &'account AccountView,
        rent_sysvar: &'account AccountView,
        space: u64,
        owner: &'address Address,
    ) -> Result<Self, ProgramError> {
        Self::with_minimum_balance(from, to, space, owner, Some(rent_sysvar))
    }

    #[inline(always)]
    pub fn with_minimum_balance(
        from: &'account AccountView,
        to: &'account AccountView,
        space: u64,
        owner: &'address Address,
        rent_sysvar: Option<&'account AccountView>,
    ) -> Result<Self, ProgramError> {
        let lamports = if let Some(rent_sysvar) = rent_sysvar {
            let rent = Rent::from_account_view(rent_sysvar)?;
            rent.try_minimum_balance(space as usize)?
        } else {
            Rent::get()?.try_minimum_balance(space as usize)?
        };

        Ok(Self {
            from,
            to,
            lamports,
            space,
            owner,
        })
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts.
        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 2];
        instruction_accounts[0].write(InstructionAccount::writable_signer(self.from.address()));
        instruction_accounts[1].write(InstructionAccount::writable_signer(self.to.address()));

        // Instruction data:
        // - [0..4  ]: instruction discriminator
        // - [4..12 ]: lamports
        // - [12..20]: account space
        // - [20..52]: owner address
        let mut instruction_data = [const { MaybeUninit::<u8>::uninit() }; 52];
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

            copy_nonoverlapping(
                self.space.to_le_bytes().as_ptr(),
                dst.add(12),
                size_of::<u64>(),
            );

            copy_nonoverlapping(self.owner.as_ref().as_ptr(), dst.add(20), ADDRESS_BYTES);
        }

        let instruction = InstructionView {
            program_id: &crate::ID,
            // SAFETY: `instruction_accounts` was initialized.
            accounts: unsafe { from_raw_parts(instruction_accounts.as_ptr() as _, 2) },
            // SAFETY: `instruction_data` was initialized.
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 52) },
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
