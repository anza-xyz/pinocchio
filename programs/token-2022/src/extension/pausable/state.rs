use core::mem::MaybeUninit;
use crate::{write_bytes, UNINIT_BYTE};
use crate::extension::consts::PAUSABLE_EXTENSION;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PausableInstruction {
    Initialize,
    Pause,
    Resume,
}

pub fn pausable_instruction_data(
    instruction_type: PausableInstruction,
) -> [MaybeUninit<u8>; 2] {
    // instruction data
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    
    let mut data = [UNINIT_BYTE; 2];
    // Set extension discriminator at offset [0]
    write_bytes(&mut data, &[PAUSABLE_EXTENSION]);
    // Set sub-instruction at offset [1]
    write_bytes(&mut data[1..2], &[instruction_type as u8]);

    data
}

pub fn pausable_initialize_instruction_data(
    instruction_type: PausableInstruction,
    authority: [u8; 32],
) -> [MaybeUninit<u8>; 34] {
    // instruction data
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    // -  [2..34]: authority (32 bytes, Pubkey)
    
    let mut data = [UNINIT_BYTE; 34];
    // Set extension discriminator at offset [0]
    write_bytes(&mut data, &[PAUSABLE_EXTENSION]);
    // Set sub-instruction at offset [1]
    write_bytes(&mut data[1..2], &[instruction_type as u8]);
    // Set authority at offset [2..34]
    write_bytes(&mut data[2..34], &authority);

    data
}
