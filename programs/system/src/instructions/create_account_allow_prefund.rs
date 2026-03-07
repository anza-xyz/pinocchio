use {
    crate::instructions::{write_bytes, UNINIT_BYTE},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_with_bounds, Signer},
        error::ProgramError,
        instruction::{InstructionAccount, InstructionView},
        sysvars::{rent::Rent, Sysvar},
        AccountView, Address, ProgramResult,
    },
};

/// Funding lamports to transfer into a newly created account.
pub struct Funding<'a> {
    /// Funding account.
    pub from: &'a AccountView,

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
/// This instruction does not warn if the account has more than
/// enough lamports. Special care must be taken to ensure that
/// the account being created has a lamports balance.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` New account
///   1. `[WRITE, SIGNER]` (OPTIONAL) Funding account
pub struct CreateAccountAllowPrefund<'a, 'b> {
    /// New account.
    pub to: &'a AccountView,

    /// Number of bytes of memory to allocate.
    pub space: u64,

    /// Address of program that will own the new account.
    pub owner: &'b Address,

    /// Funding for the new account.
    ///
    /// If `None`, the instruction will not transfer any
    /// lamports to the new account.
    pub funding: Option<Funding<'a>>,
}

impl<'a, 'b> CreateAccountAllowPrefund<'a, 'b> {
    /// Creates a new `CreateAccountAllowPrefund` instruction with the minimum
    /// balance required for the account.
    ///
    /// If the account already has lamports to cover the minimum balance, then
    /// no lamports will be transferred.
    #[inline(always)]
    pub fn with_minimum_balance(
        from: &'a AccountView,
        to: &'a AccountView,
        space: u64,
        owner: &'b Address,
        rent_sysvar: Option<&'a AccountView>,
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
            funding: Some(Funding { from, lamports }),
        })
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // instruction data
        // - [0..4  ]: instruction discriminator
        // - [4..12 ]: lamports
        // - [12..20]: account space
        // - [20..52]: owner address
        let mut instruction_data = [UNINIT_BYTE; 52];

        write_bytes(&mut instruction_data[..4], &[13, 0, 0, 0]);

        write_bytes(&mut instruction_data[12..20], &self.space.to_le_bytes());

        write_bytes(&mut instruction_data[20..52], self.owner.as_ref());

        // Determine the accounts to pass to the instruction based on whether funding
        // is present or not.

        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 2];

        instruction_accounts[0].write(InstructionAccount::writable_signer(self.to.address()));

        let mut accounts = [const { MaybeUninit::<&AccountView>::uninit() }; 2];

        accounts[0].write(self.to);

        let expected_accounts = if let Some(funding) = &self.funding {
            write_bytes(
                &mut instruction_data[4..12],
                &funding.lamports.to_le_bytes(),
            );

            instruction_accounts[1]
                .write(InstructionAccount::writable_signer(funding.from.address()));

            accounts[1].write(funding.from);

            2
        } else {
            write_bytes(&mut instruction_data[4..12], &[0; 8]);

            1
        };

        invoke_signed_with_bounds::<2, _>(
            &InstructionView {
                program_id: &crate::ID,
                // SAFETY: instruction accounts has `expected_accounts` initialized.
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                // SAFETY: instruction data is initialized.
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 52) },
            },
            // SAFETY: accounts has `expected_accounts` initialized.
            unsafe { from_raw_parts(accounts.as_ptr() as *const &AccountView, expected_accounts) },
            signers,
        )
    }
}
