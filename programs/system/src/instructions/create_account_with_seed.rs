use pinocchio::sysvars::rent::Rent;
use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    AccountRole, InstructionView,
};
use solana_program_error::{ProgramError, ProgramResult};

/// Create a new account at an address derived from a base address and a seed.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Funding account
///   1. `[WRITE]` Created account
///   2. `[SIGNER]` (optional) Base account; the account matching the base address below must be
///      provided as a signer, but may be the same as the funding account
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
        let rent = Rent::from_account_info(rent_sysvar)?;
        let lamports = rent.minimum_balance(space as usize);

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
        // account metadata
        let account_metas: [AccountRole; 3] = [
            AccountRole::writable_signer(self.from.address()),
            AccountRole::writable(self.to.address()),
            AccountRole::readonly_signer(self.base.unwrap_or(self.from).address()),
        ];

        // instruction data
        // - [0..4  ]: instruction discriminator
        // - [4..36 ]: base address
        // - [36..44]: seed length
        // - [44..  ]: seed (max 32)
        // - [..  +8]: lamports
        // - [..  +8]: account space
        // - [.. +32]: owner address
        let mut instruction_data = [0; 120];
        instruction_data[0] = 3;
        instruction_data[4..36]
            .copy_from_slice(self.base.unwrap_or(self.from).address().as_array());
        instruction_data[36..44].copy_from_slice(&u64::to_le_bytes(self.seed.len() as u64));

        let offset = 44 + self.seed.len();
        instruction_data[44..offset].copy_from_slice(self.seed.as_bytes());
        instruction_data[offset..offset + 8].copy_from_slice(&self.lamports.to_le_bytes());
        instruction_data[offset + 8..offset + 16].copy_from_slice(&self.space.to_le_bytes());
        instruction_data[offset + 16..offset + 48].copy_from_slice(self.owner.as_ref());

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data[..offset + 48],
        };

        invoke_signed(
            &instruction,
            &[self.from, self.to, self.base.unwrap_or(self.from)],
            signers,
        )
    }
}
