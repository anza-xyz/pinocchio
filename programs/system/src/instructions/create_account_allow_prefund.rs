use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::rent::Rent,
    ProgramResult,
};

/// Create a new account with the `lamports==0` assertion.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` New account
///   1. `[WRITE, SIGNER]` (OPTIONAL) Funding account
pub struct CreateAccountAllowPrefund<'a> {
    /// Funding account and number of lamports to transfer to the new account.
    pub payer_and_lamports: Option<(&'a AccountInfo, u64)>,

    /// New account.
    pub to: &'a AccountInfo,

    /// Number of bytes of memory to allocate.
    pub space: u64,

    /// Address of program that will own the new account.
    pub owner: &'a Pubkey,
}

impl<'a> CreateAccountAllowPrefund<'a> {
    #[inline(always)]
    pub fn with_minimal_balance(
        from: &'a AccountInfo,
        to: &'a AccountInfo,
        rent_sysvar: &'a AccountInfo,
        space: u64,
        owner: &'a Pubkey,
    ) -> Result<Self, ProgramError> {
        let rent = Rent::from_account_info(rent_sysvar)?;
        let lamports = rent.minimum_balance(space as usize);

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
        // - [20..52]: owner pubkey
        let mut instruction_data = [0; 52];
        // create account instruction has a '0' discriminator. Lamports remains 0 here
        // but may be changed later.
        instruction_data[12..20].copy_from_slice(&self.space.to_le_bytes());
        instruction_data[20..52].copy_from_slice(self.owner.as_ref());

        if let Some((payer, lamports)) = self.payer_and_lamports {
            instruction_data[4..12].copy_from_slice(&lamports.to_le_bytes());
            let account_metas = [
                AccountMeta::writable_signer(self.to.key()),
                AccountMeta::writable_signer(payer.key()),
            ];
            let instruction = Instruction {
                program_id: &crate::ID,
                accounts: &account_metas,
                data: &instruction_data,
            };
            invoke_signed(&instruction, &[self.to, payer], signers)
        } else {
            let account_metas = [AccountMeta::writable_signer(self.to.key())];
            let instruction = Instruction {
                program_id: &crate::ID,
                accounts: &account_metas,
                data: &instruction_data,
            };
            invoke_signed(&instruction, &[self.to], signers)
        }
    }
}
