use {
    pinocchio::{
        account_info::AccountInfo,
        instruction::{AccountMeta, Instruction, Signer},
        pubkey::Pubkey,
        ProgramResult,
    },
    pinocchio_cpi::invoke_signed,
};

/// Create a new account at an address derived from a base pubkey and a seed.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Funding account
///   1. `[WRITE]` Created account
///   2. `[SIGNER]` (optional) Base account; the account matching the base Pubkey below must be
///      provided as a signer, but may be the same as the funding account
pub struct CreateAccountWithSeed<'a, 'b, 'c> {
    /// Funding account.
    pub from: &'a AccountInfo,

    /// New account.
    pub to: &'a AccountInfo,

    /// Base account.
    ///
    /// The account matching the base Pubkey below must be provided as
    /// a signer, but may be the same as the funding account and provided
    /// as account 0.
    pub base: Option<&'a AccountInfo>,

    /// String of ASCII chars, no longer than `Pubkey::MAX_SEED_LEN`.
    pub seed: &'b str,

    /// Number of lamports to transfer to the new account.
    pub lamports: u64,

    /// Number of bytes of memory to allocate.
    pub space: u64,

    /// Address of program that will own the new account.
    pub owner: &'c Pubkey,
}

impl CreateAccountWithSeed<'_, '_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable_signer(self.from.key()),
            AccountMeta::writable(self.to.key()),
            AccountMeta::readonly_signer(self.base.unwrap_or(self.from).key()),
        ];

        // instruction data
        // - [0..4  ]: instruction discriminator
        // - [4..36 ]: base pubkey
        // - [36..40]: seed length
        // - [40..  ]: seed (max 32)
        // - [..  +8]: lamports
        // - [..  +8]: account space
        // - [.. +32]: owner pubkey
        let mut instruction_data = [0; 120];
        instruction_data[0] = 3;
        instruction_data[4..36].copy_from_slice(self.base.unwrap_or(self.from).key());
        instruction_data[36..40].copy_from_slice(&u32::to_le_bytes(self.seed.len() as u32));

        let offset = 40 + self.seed.len();
        instruction_data[40..offset].copy_from_slice(self.seed.as_bytes());
        instruction_data[offset..offset + 8].copy_from_slice(&self.lamports.to_le_bytes());
        instruction_data[offset + 8..offset + 16].copy_from_slice(&self.space.to_le_bytes());
        instruction_data[offset + 16..offset + 48].copy_from_slice(self.owner.as_ref());

        let instruction = Instruction {
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
