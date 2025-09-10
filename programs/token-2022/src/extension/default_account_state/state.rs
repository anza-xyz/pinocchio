use core::mem::MaybeUninit;

use crate::extension::consts::DEFAULT_ACCOUNT_STATE_EXTENSION;

/// Default Account State extension instructions
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum DefaultAccountStateInstruction {
    /// Initialize a new mint with the default state for new Accounts
    Initialize = 0,
    /// Update the default state for new Accounts
    Update = 1,
}

/// Helper function to write bytes to MaybeUninit array
#[inline(always)]
fn write_bytes(dst: &mut [MaybeUninit<u8>], src: &[u8]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        d.write(*s);
    }
}

pub fn default_account_state_instruction_data(
    instruction_type: DefaultAccountStateInstruction,
    state: u8,
) -> [MaybeUninit<u8>; 3] {
    // instruction data
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    // -  [2]: state (1 byte, u8)
    
    const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::uninit();
    let mut data = [UNINIT_BYTE; 3];
    // Set extension discriminator at offset [0]
    write_bytes(&mut data, &[DEFAULT_ACCOUNT_STATE_EXTENSION]);
    // Set sub-instruction at offset [1]
    write_bytes(&mut data[1..2], &[instruction_type as u8]);
    // Set state at offset [2]
    write_bytes(&mut data[2..3], &[state]);

    data
}
