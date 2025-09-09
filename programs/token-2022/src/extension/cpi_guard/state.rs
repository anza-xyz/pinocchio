use core::mem::MaybeUninit;
use crate::{write_bytes, UNINIT_BYTE};
use crate::extension::consts::CPI_GUARD_EXTENSION;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CpiGuardInstruction {
    Enable,
    Disable,
}

pub fn cpi_guard_instruction_data(
    instruction_type: CpiGuardInstruction,
) -> [MaybeUninit<u8>; 2] {
    // instruction data
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    
    let mut data = [UNINIT_BYTE; 2];
    // Set extension discriminator at offset [0]
    write_bytes(&mut data, &[CPI_GUARD_EXTENSION]);
    // Set sub-instruction at offset [1]
    write_bytes(&mut data[1..2], &[instruction_type as u8]);

    data
}