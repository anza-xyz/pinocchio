use pinocchio::{
    cpi::{invoke_signed, Signer},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    sysvars::rent::Rent,
    AccountView, Address, ProgramResult,
};

/// Create a new account without the `lamports==0` assertion.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` New account
///   1. `[WRITE, SIGNER]` (OPTIONAL) Funding account
pub struct CreateAccountAllowPrefund<'a> {
    /// Funding account and number of lamports to transfer to the new account.
    pub payer_and_lamports: Option<(&'a AccountView, u64)>,

    /// New account.
    pub to: &'a AccountView,

    /// Number of bytes of memory to allocate.
    pub space: u64,

    /// Address of program that will own the new account.
    pub owner: &'a Address,
}

impl<'a> CreateAccountAllowPrefund<'a> {
    #[inline(always)]
    pub fn with_minimal_balance(
        from: &'a AccountView,
        to: &'a AccountView,
        rent_sysvar: &'a AccountView,
        space: u64,
        owner: &'a Address,
    ) -> Result<Self, ProgramError> {
        let rent = Rent::from_account_view(rent_sysvar)?;
        let lamports = rent.try_minimum_balance(space as usize)?;

        Ok(Self {
            payer_and_lamports: Some((from, lamports)),
            to,
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
        // instruction data
        // - [0..4  ]: instruction discriminator
        // - [4..12 ]: lamports
        // - [12..20]: account space
        // - [20..52]: owner address
        let mut instruction_data = [0; 52];
        // create account instruction has a '0' discriminator. Lamports remains 0 here
        // but may be changed later.
        instruction_data[12..20].copy_from_slice(&self.space.to_le_bytes());
        instruction_data[20..52].copy_from_slice(self.owner.as_ref());

        if let Some((payer, lamports)) = self.payer_and_lamports {
            instruction_data[4..12].copy_from_slice(&lamports.to_le_bytes());
            let instruction_accounts: [InstructionAccount; 2] = [
                InstructionAccount::writable_signer(self.to.address()),
                InstructionAccount::writable_signer(payer.address()),
            ];
            let instruction = InstructionView {
                program_id: &crate::ID,
                accounts: &instruction_accounts,
                data: &instruction_data,
            };
            invoke_signed(&instruction, &[self.to, payer], signers)
        } else {
            let instruction_accounts: [InstructionAccount; 1] =
                [InstructionAccount::writable_signer(self.to.address())];
            let instruction = InstructionView {
                program_id: &crate::ID,
                accounts: &instruction_accounts,
                data: &instruction_data,
            };
            invoke_signed(&instruction, &[self.to], signers)
        }
    }
}
