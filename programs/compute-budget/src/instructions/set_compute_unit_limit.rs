use pinocchio::{instruction::Instruction, program::invoke, ProgramResult};

/// Set compute unit limit for a transaction.
///
/// Limits the compute units consumed. Useful to reduce fees when you know
/// your program uses less than the default 1.4M CU limit.
///
/// # Example
///
/// ```ignore
/// SetComputeUnitLimit {
///     units: 50_000,
/// }.invoke()?;
/// ```
pub struct SetComputeUnitLimit {
    pub units: u32,
}

impl SetComputeUnitLimit {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {

        let mut instruction_data = [0u8; 5];
        instruction_data[0] = 2;
        instruction_data[1..5].copy_from_slice(&self.units.to_le_bytes());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &[],
            data: &instruction_data,
        };

        invoke(&instruction, &[])
    }
}
