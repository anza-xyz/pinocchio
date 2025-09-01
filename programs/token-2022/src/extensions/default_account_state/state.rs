use crate::{state::AccountState, write_bytes, UNINIT_BYTE};
use core::mem::MaybeUninit;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DefaultAccountStateInstruction {
    Initialize,
    Update,
}

/// Discriminator for the DefaultAccountState extension.
const DEFAULT_ACCOUNT_STATE_EXTENSION: u8 = 28;

/// Packs instruction data for a `DefaultAccountStateInstruction`
pub fn encode_instruction_data(
    instruction_type: DefaultAccountStateInstruction,
    state: AccountState,
) -> [MaybeUninit<u8>; 3] {
    // instruction data
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    // -  [1]: account state (1 byte, u8)
    let mut data = [UNINIT_BYTE; 3];
    // Set discriminator as u8 at offset [0]
    write_bytes(&mut data, &[DEFAULT_ACCOUNT_STATE_EXTENSION]);
    // Set instruction_type as u8 at offset [1]
    write_bytes(&mut data[1..2], &[instruction_type as u8]);
    // Set account state as u8 at offset [2]
    write_bytes(&mut data[2..3], &[state as u8]);
    data
}
