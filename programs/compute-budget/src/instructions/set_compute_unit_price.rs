use pinocchio::{instruction::Instruction, program::invoke, ProgramResult};

/// Set compute unit price for transaction prioritization.
///
/// Higher prices lead to faster confirmation. Priority fee is calculated as:
/// `priority_fee = compute_unit_limit * compute_unit_price`
///
/// # Example
///
/// ```ignore
/// SetComputeUnitPrice {
///     micro_lamports: 10_000,
/// }.invoke()?;
/// ```
pub struct SetComputeUnitPrice {

    pub micro_lamports: u64,
}

impl SetComputeUnitPrice {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {

        let mut instruction_data = [0u8; 9];
        instruction_data[0] = 3;
        instruction_data[1..9].copy_from_slice(&self.micro_lamports.to_le_bytes());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &[],
            data: &instruction_data,
        };

        invoke(&instruction, &[])
    }
}
