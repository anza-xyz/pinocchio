use pinocchio::{
    cpi::{invoke_signed, Signer},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};

/// Create a new account without the `lamports==0` assertion.
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

    /// Funding account and lamports to transfer to the new account.
    pub payer_and_lamports: Option<(&'a AccountView, u64)>,
}

impl<'a, 'b> CreateAccountAllowPrefund<'a, 'b> {
    #[inline(always)]
    /// Creates a new `CreateAccountAllowPrefund` instruction with the minimum balance required
    /// for the account. The caller must provide a `payer` if the account needs lamports;
    /// otherwise, the resulting instruction will fail (downstream) when invoked.
    ///
    /// This instruction does not warn if the account has more than enough lamports; large
    /// lamport balances can be frozen by `CreateAccountAllowPrefund` if used incorrectly.
    pub fn with_minimum_balance(
        to: &'a AccountView,
        space: u64,
        owner: &'b Address,
        payer: Option<&'a AccountView>,
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
            payer_and_lamports: payer.map(|p| (p, lamports)),
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
        let mut instruction_data = [0u8; 52];
        // CreateAccountAllowPrefund has discriminator 13
        instruction_data[0] = 13;
        // Lamports remains 0 here, but may be changed just below
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
