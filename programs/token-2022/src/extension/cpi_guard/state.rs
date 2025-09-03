use crate::extension::consts::CPI_GUARD_EXTENSION;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CpiGuardInstruction {
    Enable,
    Disable,
}

pub fn cpi_guard_instruction_data(
    instruction_type: CpiGuardInstruction,
) -> [u8; 2] {
    // instruction data
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    let mut data = [0u8; 2];
    data[0] = CPI_GUARD_EXTENSION;
    data[1] = instruction_type as u8;
    data
}