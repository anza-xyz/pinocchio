use pinocchio::{
    cpi::{invoke_signed, Signer},
    instruction::{InstructionAccount, InstructionView},
    AccountView, Address, ProgramResult,
};

/// Assign account to a program based on a seed.
///
/// ### Accounts:
///   0. `[WRITE]` Assigned account
///   1. `[SIGNER]` Base account
pub struct AssignWithSeed<'a, 'b, 'c> {
    /// Allocated account.
    pub account: &'a AccountView,

    /// Base account.
    ///
    /// The account matching the base `Address` below must be provided as
    /// a signer, but may be the same as the funding account and provided
    /// as account 0.
    pub base: &'a AccountView,

    /// String of ASCII chars, no longer than [`MAX_SEED_LEN`](https://docs.rs/solana-address/latest/solana_address/constant.MAX_SEED_LEN.html).
    pub seed: &'b str,

    /// Address of program that will own the new account.
    pub owner: &'c Address,
}

impl AssignWithSeed<'_, '_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts
        let instruction_accounts: [InstructionAccount; 2] = [
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::readonly_signer(self.base.address()),
        ];

        // instruction data
        // - [0..4  ]: instruction discriminator
        // - [4..36 ]: base address
        // - [36..44]: seed length
        // - [44..  ]: seed (max 32)
        // - [.. +32]: owner address
        let mut instruction_data = [0; 104];
        instruction_data[0] = 10;
        instruction_data[4..36].copy_from_slice(self.base.address().as_array());
        instruction_data[36..44].copy_from_slice(&u64::to_le_bytes(self.seed.len() as u64));

        let offset = 44 + self.seed.len();
        instruction_data[44..offset].copy_from_slice(self.seed.as_bytes());
        instruction_data[offset..offset + 32].copy_from_slice(self.owner.as_ref());

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            data: &instruction_data[..offset + 32],
        };

        invoke_signed(&instruction, &[self.account, self.base], signers)
    }
}
