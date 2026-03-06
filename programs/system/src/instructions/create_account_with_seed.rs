use {
    crate::instructions::{write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    pinocchio::{
        address::MAX_SEED_LEN,
        cpi::{invoke_signed, Signer},
        error::ProgramError,
        instruction::{InstructionAccount, InstructionView},
        sysvars::{rent::Rent, Sysvar},
        AccountView, Address, ProgramResult,
    },
};

/// Create a new account at an address derived from a base address and a seed.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Funding account
///   1. `[WRITE]` Created account
///   2. `[SIGNER]` (optional) Base account; the account matching the base
///      address below must be provided as a signer, but may be the same as the
///      funding account
pub struct CreateAccountWithSeed<'a, 'b, 'c> {
    /// Funding account.
    pub from: &'a AccountView,

    /// New account.
    pub to: &'a AccountView,

    /// Base account.
    ///
    /// The account matching the base [`Address`] below must be provided as
    /// a signer, but may be the same as the funding account and provided
    /// as account 0.
    pub base: Option<&'a AccountView>,

    /// String of ASCII chars, no longer than [`MAX_SEED_LEN`](https://docs.rs/solana-address/latest/solana_address/constant.MAX_SEED_LEN.html).
    pub seed: &'b str,

    /// Number of lamports to transfer to the new account.
    pub lamports: u64,

    /// Number of bytes of memory to allocate.
    pub space: u64,

    /// Address of program that will own the new account.
    pub owner: &'c Address,
}

impl<'a, 'b, 'c> CreateAccountWithSeed<'a, 'b, 'c> {
    #[deprecated(since = "0.5.0", note = "Use `with_minimum_balance` instead")]
    #[inline(always)]
    pub fn with_minimal_balance(
        from: &'a AccountView,
        to: &'a AccountView,
        base: Option<&'a AccountView>,
        seed: &'b str,
        rent_sysvar: &'a AccountView,
        space: u64,
        owner: &'c Address,
    ) -> Result<Self, ProgramError> {
        Self::with_minimum_balance(from, to, base, seed, space, owner, Some(rent_sysvar))
    }

    #[inline(always)]
    pub fn with_minimum_balance(
        from: &'a AccountView,
        to: &'a AccountView,
        base: Option<&'a AccountView>,
        seed: &'b str,
        space: u64,
        owner: &'c Address,
        rent_sysvar: Option<&'a AccountView>,
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
            base,
            seed,
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
        // Instruction accounts
        let instruction_accounts: [InstructionAccount; 3] = [
            InstructionAccount::writable_signer(self.from.address()),
            InstructionAccount::writable(self.to.address()),
            InstructionAccount::readonly_signer(self.base.unwrap_or(self.from).address()),
        ];

        if self.seed.len() > MAX_SEED_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        // instruction data
        // - [0..4  ]: instruction discriminator
        // - [4..36 ]: base address
        // - [36..44]: seed length
        // - [44..  ]: seed (max 32)
        // - [..  +8]: lamports
        // - [..  +8]: account space
        // - [.. +32]: owner address
        let mut instruction_data = [UNINIT_BYTE; 124];

        write_bytes(&mut instruction_data[..4], &[3, 0, 0, 0]);

        write_bytes(
            &mut instruction_data[4..36],
            self.base.unwrap_or(self.from).address().as_array(),
        );

        write_bytes(
            &mut instruction_data[36..44],
            &u64::to_le_bytes(self.seed.len() as u64),
        );

        let offset = 44 + self.seed.len();
        write_bytes(
            // SAFETY: instruction data allocated `MAX_SEED_LEN` bytes
            // for the seed.
            unsafe { instruction_data.get_unchecked_mut(44..offset) },
            self.seed.as_bytes(),
        );

        write_bytes(
            // SAFETY: instruction data allocated space for the lamports.
            unsafe { instruction_data.get_unchecked_mut(offset..offset + 8) },
            &self.lamports.to_le_bytes(),
        );

        write_bytes(
            // SAFETY: instruction data allocated space for the `space`.
            unsafe { instruction_data.get_unchecked_mut(offset + 8..offset + 16) },
            &self.space.to_le_bytes(),
        );

        write_bytes(
            // SAFETY: instruction data allocated space for the owner address.
            unsafe { instruction_data.get_unchecked_mut(offset + 16..offset + 48) },
            self.owner.as_ref(),
        );

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            // SAFETY: The instruction data is initialized.
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as *const _, offset + 48) },
        };

        invoke_signed(
            &instruction,
            &[self.from, self.to, self.base.unwrap_or(self.from)],
            signers,
        )
    }
}
