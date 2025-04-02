use pinocchio::{
    account_view::AccountView,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    Address, ProgramResult,
};

/// Allocate space for and assign an account at an address derived
/// from a base address and a seed.
///
/// ### Accounts:
///   0. `[WRITE]` Allocated account
///   1. `[SIGNER]` Base account
pub struct AllocateWithSeed<'a, 'b, 'c> {
    /// Allocated account.
    pub account: &'a AccountView,

    /// Base account.
    ///
    /// The account matching the base address below must be provided as
    /// a signer, but may be the same as the funding account and provided
    /// as account 0.
    pub base: &'a AccountView,

    /// String of ASCII chars, no longer than [`solana_address::MAX_SEED_LEN`].
    pub seed: &'b str,

    /// Number of bytes of memory to allocate.
    pub space: u64,

    /// Address of program that will own the new account.
    pub owner: &'c Address,
}

impl AllocateWithSeed<'_, '_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly_signer(self.base.key()),
        ];

        // instruction data
        // - [0..4  ]: instruction discriminator
        // - [4..36 ]: base address
        // - [36..44]: seed length
        // - [44..  ]: seed (max 32)
        // - [..  +8]: account space
        // - [.. +32]: owner address
        let mut instruction_data = [0; 112];
        instruction_data[0] = 9;
        instruction_data[4..36].copy_from_slice(self.base.key().as_array());
        instruction_data[36..44].copy_from_slice(&u64::to_le_bytes(self.seed.len() as u64));

        let offset = 44 + self.seed.len();
        instruction_data[44..offset].copy_from_slice(self.seed.as_bytes());
        instruction_data[offset..offset + 8].copy_from_slice(&self.space.to_le_bytes());
        instruction_data[offset + 8..offset + 40].copy_from_slice(self.owner.as_ref());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data[..offset + 40],
        };

        invoke_signed(&instruction, &[self.account, self.base], signers)
    }
}
