use {
    crate::instructions::{write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    pinocchio::{
        address::MAX_SEED_LEN,
        cpi::{invoke_signed, Signer},
        error::ProgramError,
        instruction::{InstructionAccount, InstructionView},
        AccountView, Address, ProgramResult,
    },
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

    /// String of ASCII chars, no longer than [`MAX_SEED_LEN`](https://docs.rs/solana-address/latest/solana_address/constant.MAX_SEED_LEN.html).
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
        // Instruction accounts
        let instruction_accounts: [InstructionAccount; 2] = [
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::readonly_signer(self.base.address()),
        ];

        if self.seed.len() > MAX_SEED_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        // instruction data
        // - [0..4  ]: instruction discriminator
        // - [4..36 ]: base address
        // - [36..44]: seed length
        // - [44..  ]: seed (max 32)
        // - [..  +8]: account space
        // - [.. +32]: owner address
        let mut instruction_data = [UNINIT_BYTE; 116];

        instruction_data[0].write(9);
        instruction_data[1].write(0);
        instruction_data[2].write(0);
        instruction_data[3].write(0);

        write_bytes(&mut instruction_data[4..36], self.base.address().as_array());

        write_bytes(
            &mut instruction_data[36..44],
            &u64::to_le_bytes(self.seed.len() as u64),
        );

        let offset = 44 + self.seed.len();
        write_bytes(&mut instruction_data[44..offset], self.seed.as_bytes());

        write_bytes(
            &mut instruction_data[offset..offset + 8],
            &self.space.to_le_bytes(),
        );

        write_bytes(
            &mut instruction_data[offset + 8..offset + 40],
            self.owner.as_ref(),
        );

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            // SAFETY: The instruction data is initialized.
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as *const _, offset + 40) },
        };

        invoke_signed(&instruction, &[self.account, self.base], signers)
    }
}
