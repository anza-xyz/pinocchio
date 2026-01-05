use pinocchio::{
    cpi::{invoke_signed, Signer},
    instruction::{InstructionAccount, InstructionView},
    AccountView, ProgramResult,
};

/// Allocate space in a (possibly new) account without funding.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` New account
pub struct Allocate<'a> {
    /// Account to be assigned.
    pub account: &'a AccountView,

    /// Number of bytes of memory to allocate.
    pub space: u64,
}

impl Allocate<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts
        let instruction_accounts: [InstructionAccount; 1] =
            [InstructionAccount::writable_signer(self.account.address())];

        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..12]: space
        let mut instruction_data = [0; 12];
        instruction_data[0] = 8;
        instruction_data[4..12].copy_from_slice(&self.space.to_le_bytes());

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            data: &instruction_data,
        };

        invoke_signed(&instruction, &[self.account], signers)
    }
}
