use pinocchio::{
    cpi::invoke_signed,
    instruction::{Instruction, Signer},
    ProgramResult,
};

/// Get the minimum delegation amount for a stake account.
///
/// ### Accounts:
pub struct GetMinimumDelegation {}

impl GetMinimumDelegation {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data
        let instruction_data = [13u8; 1];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &[],
            data: &instruction_data,
        };

        invoke_signed(
            &instruction,
            &[],
            signers,
        )
    }
}
