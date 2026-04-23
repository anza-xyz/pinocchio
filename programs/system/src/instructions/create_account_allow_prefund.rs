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

/// Funding lamports to transfer into a newly created account.
pub struct Funding<'account> {
    /// Funding account.
    pub from: &'account AccountView,

    /// Number of lamports to transfer to the new account.
    pub lamports: u64,
}

/// Create a new account allowing the account to be prefunded.
///
/// This instruction is identical to `CreateAccount` except
/// that it allows the account being created to already have
/// lamports in it.
///
/// # Important
///
/// Special care should be taken not to accidentally use a wallet
/// account as the new account. This instruction allocates space
/// and assigns the account to a new program owner, which makes
/// the account unusable as a wallet. No warning is given if the
/// account has more than enough lamports.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` New account
///   1. `[WRITE, SIGNER]` (optional) Funding account
pub struct CreateAccountAllowPrefund<'account, 'address> {
    /// New account.
    pub to: &'account AccountView,

    /// Number of bytes of memory to allocate.
    pub space: u64,

    /// Address of program that will own the new account.
    pub owner: &'address Address,

    /// Funding for the new account.
    ///
    /// If `None`, the instruction will not transfer any
    /// lamports to the new account.
    pub funding: Option<Funding<'account>>,
}

impl<'account, 'address> CreateAccountAllowPrefund<'account, 'address> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u32 = 13;

    /// Creates a new `CreateAccountAllowPrefund` instruction with the minimum
    /// balance required for the account.
    ///
    /// If the account already has lamports to cover the minimum balance, then
    /// no lamports will be transferred.
    #[inline(always)]
    pub fn with_minimum_balance(
        from: &'account AccountView,
        to: &'account AccountView,
        space: u64,
        owner: &'address Address,
        rent_sysvar: Option<&'account AccountView>,
    ) -> Result<Self, ProgramError> {
        let required_lamports = if let Some(rent_sysvar) = rent_sysvar {
            Rent::from_account_view(rent_sysvar)?.try_minimum_balance(space as usize)?
        } else {
            Rent::get()?.try_minimum_balance(space as usize)?
        };

        let lamports = required_lamports.saturating_sub(to.lamports());

        Ok(Self {
            to,
            space,
            owner,
            funding: if lamports == 0 {
                None
            } else {
                Some(Funding { from, lamports })
            },
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
        instruction_accounts[0].write(InstructionAccount::writable_signer(self.to.address()));

        // instruction data
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
                self.space.to_le_bytes().as_ptr(),
                dst.add(12),
                size_of::<u64>(),
            );

            copy_nonoverlapping(self.owner.as_ref().as_ptr(), dst.add(20), ADDRESS_BYTES);
        }

        let mut accounts = [const { MaybeUninit::<CpiAccount>::uninit() }; 2];
        CpiAccount::init_from_account_view(self.to, &mut accounts[0]);

        // Determine the accounts to pass to the instruction based on whether funding
        // is present or not.
        let expected_accounts = if let Some(funding) = &self.funding {
            if self.to.is_borrowed() | funding.from.is_borrowed() {
                return Err(ProgramError::AccountBorrowFailed);
            }

            // SAFETY: The copy is within bounds of the allocated data.
            unsafe {
                let dst = instruction_data.as_mut_ptr() as *mut u8;

                copy_nonoverlapping(
                    funding.lamports.to_le_bytes().as_ptr(),
                    dst.add(4),
                    size_of::<u64>(),
                );
            }

            instruction_accounts[1]
                .write(InstructionAccount::writable_signer(funding.from.address()));
            CpiAccount::init_from_account_view(funding.from, &mut accounts[1]);

            2
        } else {
            if self.to.is_borrowed() {
                return Err(ProgramError::AccountBorrowFailed);
            }

            // SAFETY: The copy is within bounds of the allocated data.
            unsafe {
                let dst = instruction_data.as_mut_ptr() as *mut u8;
                copy_nonoverlapping(0u64.to_le_bytes().as_ptr(), dst.add(4), size_of::<u64>());
            }

            1
        };

        // SAFETY: `accounts` was initialized and not borrowed.
        unsafe {
            invoke_signed_unchecked(
                &InstructionView {
                    program_id: &crate::ID,
                    accounts: from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts),
                    data: from_raw_parts(instruction_data.as_ptr() as _, 52),
                },
                from_raw_parts(accounts.as_ptr() as _, expected_accounts),
                signers,
            )
        };

        Ok(())
    }
}
