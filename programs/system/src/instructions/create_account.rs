use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    sysvars::rent::Rent,
    program_error::ProgramError,
    ProgramResult,
};

/// Create a new account.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Funding account
///   1. `[WRITE, SIGNER]` New account
pub struct CreateAccount<'a> {
    /// Funding account.
    pub from: &'a AccountInfo,

    /// New account.
    pub to: &'a AccountInfo,

    /// Number of lamports to transfer to the new account.
    pub lamports: u64,

    /// Number of bytes of memory to allocate.
    pub space: u64,

    /// Address of program that will own the new account.
    pub owner: &'a Pubkey,
}

impl<'a> CreateAccount<'a> {
    pub fn with_rent_check(
        from: &'a AccountInfo,
        to: &'a AccountInfo,
        rent_sysvar: &'a AccountInfo,
        space: u64,        
        owner: &'a Pubkey,
    ) -> Result<Self, ProgramError> {
        let rent = Rent::from_account_info(rent_sysvar)?;
        let lamports = rent.minimum_balance(space as usize);

        if from.lamports() < lamports {
            return Err(ProgramError::InsufficientFunds);
        }

        if !to.data_is_empty() {
            return Err(ProgramError::InvalidAccountData);
        }

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
        // account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable_signer(self.from.key()),
            AccountMeta::writable_signer(self.to.key()),
        ];

        // instruction data
        // - [0..4  ]: instruction discriminator
        // - [4..12 ]: lamports
        // - [12..20]: account space
        // - [20..52]: owner pubkey
        let mut instruction_data = [0; 52];
        // create account instruction has a '0' discriminator
        instruction_data[4..12].copy_from_slice(&self.lamports.to_le_bytes());
        instruction_data[12..20].copy_from_slice(&self.space.to_le_bytes());
        instruction_data[20..52].copy_from_slice(self.owner.as_ref());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(&instruction, &[self.from, self.to], signers)
    }
}
