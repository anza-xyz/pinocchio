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

/// Transfer lamports from a derived address.
///
/// ### Accounts:
///   0. `[WRITE]` Funding account
///   1. `[SIGNER]` Base for funding account
///   2. `[WRITE]` Recipient account
pub struct TransferWithSeed<'a, 'b, 'c> {
    /// Funding account.
    pub from: &'a AccountView,

    /// Base account.
    ///
    /// The account matching the base [`Address`] below must be provided as
    /// a signer, but may be the same as the funding account and provided
    /// as account 0.
    pub base: &'a AccountView,

    /// Recipient account.
    pub to: &'a AccountView,

    /// Amount of lamports to transfer.
    pub lamports: u64,

    /// String of ASCII chars, no longer than [`MAX_SEED_LEN`](https://docs.rs/solana-address/latest/solana_address/constant.MAX_SEED_LEN.html).
    pub seed: &'b str,

    /// Address of program that will own the new account.
    pub owner: &'c Address,
}

impl TransferWithSeed<'_, '_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts
        let instruction_accounts: [InstructionAccount; 3] = [
            InstructionAccount::writable(self.from.address()),
            InstructionAccount::readonly_signer(self.base.address()),
            InstructionAccount::writable(self.to.address()),
        ];

        if self.seed.len() > MAX_SEED_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        // instruction data
        // - [0..4  ]: instruction discriminator
        // - [4..12 ]: lamports amount
        // - [12..20]: seed length
        // - [20..  ]: seed (max 32)
        // - [.. +32]: owner address
        let mut instruction_data = [UNINIT_BYTE; 84];

        write_bytes(&mut instruction_data[..4], &[11, 0, 0, 0]);

        write_bytes(&mut instruction_data[4..12], &self.lamports.to_le_bytes());

        write_bytes(
            &mut instruction_data[12..20],
            &u64::to_le_bytes(self.seed.len() as u64),
        );

        let offset = 20 + self.seed.len();
        write_bytes(
            // SAFETY: instruction data allocated `MAX_SEED_LEN` bytes
            // for the seed.
            unsafe { instruction_data.get_unchecked_mut(20..offset) },
            self.seed.as_bytes(),
        );

        write_bytes(
            // SAFETY: instruction data allocated space for the owner address.
            unsafe { instruction_data.get_unchecked_mut(offset..offset + 32) },
            self.owner.as_ref(),
        );

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            // SAFETY: The instruction data is initialized.
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as *const _, offset + 32) },
        };

        invoke_signed(&instruction, &[self.from, self.base, self.to], signers)
    }
}
