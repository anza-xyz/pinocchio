use core::mem::MaybeUninit;
use crate::{write_bytes, UNINIT_BYTE};
use crate::extension::consts::SCALED_UI_AMOUNT_EXTENSION;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScaledUiAmountInstruction {
    Initialize,
    UpdateMultiplier,
}

pub fn scaled_ui_amount_instruction_data(
    instruction_type: ScaledUiAmountInstruction,
) -> [MaybeUninit<u8>; 2] {
    // instruction data
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    
    let mut data = [UNINIT_BYTE; 2];
    // Set extension discriminator at offset [0]
    write_bytes(&mut data, &[SCALED_UI_AMOUNT_EXTENSION]);
    // Set sub-instruction at offset [1]
    write_bytes(&mut data[1..2], &[instruction_type as u8]);

    data
}

pub fn scaled_ui_amount_update_multiplier_instruction_data(
    instruction_type: ScaledUiAmountInstruction,
    multiplier: f64,
    effective_timestamp: i64,
) -> [MaybeUninit<u8>; 18] {
    // instruction data
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    // -  [2..10]: multiplier (8 bytes, f64)
    // -  [10..18]: effective_timestamp (8 bytes, i64)
    
    let mut data = [UNINIT_BYTE; 18];
    // Set extension discriminator at offset [0]
    write_bytes(&mut data, &[SCALED_UI_AMOUNT_EXTENSION]);
    // Set sub-instruction at offset [1]
    write_bytes(&mut data[1..2], &[instruction_type as u8]);
    // Set multiplier at offset [2..10]
    write_bytes(&mut data[2..10], &multiplier.to_le_bytes());
    // Set effective_timestamp at offset [10..18]
    write_bytes(&mut data[10..18], &effective_timestamp.to_le_bytes());

    data
}
