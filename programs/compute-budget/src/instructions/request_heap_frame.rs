use pinocchio::{instruction::Instruction, program::invoke, ProgramResult};

/// Request additional heap frame for the transaction.
///
/// Programs have 32 KB heap by default. Use this to request up to 256 KB total.
/// Must be a multiple of 8 KB.
///
/// # Example
///
/// ```ignore
/// RequestHeapFrame {
///     bytes: 32 * 1024,  // Request additional 32 KB
/// }.invoke()?;
/// ```
pub struct RequestHeapFrame {
    /// Additional bytes to request (must be multiple of 8 KB).
    pub bytes: u32,
}

impl RequestHeapFrame {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        
        let mut instruction_data = [0u8; 5];
        instruction_data[0] = 1;
        instruction_data[1..5].copy_from_slice(&self.bytes.to_le_bytes());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &[],
            data: &instruction_data,
        };

        invoke(&instruction, &[])
    }
}
