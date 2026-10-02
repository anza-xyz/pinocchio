use {
    crate::{write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Emits the `TokenMetadata` entry as return data.
///
/// The emitted bytes follow the `TokenMetadata` layout, sliced by the
/// optional `start` and `end` offsets. Return data can be fetched using
/// `sol_get_return_data`.
///
/// Accounts expected by this instruction:
///
///   0. `[]` The metadata account.
pub struct Emit<'a, 'b> {
    /// The metadata account.
    pub metadata: &'a AccountView,

    /// Start of the range of data to emit.
    pub start: Option<u64>,

    /// End of the range of data to emit.
    pub end: Option<u64>,

    /// The token program.
    pub token_program: &'b Address,
}

impl Emit<'_, '_> {
    /// Hash of `spl_token_metadata_interface:emitter`.
    pub const DISCRIMINATOR: [u8; 8] = [250, 166, 180, 250, 13, 12, 184, 70];

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 26];

        write_bytes(&mut instruction_data[0..8], &Self::DISCRIMINATOR);

        let mut expected_data = 8;

        for value in [self.start, self.end] {
            if let Some(value) = value {
                instruction_data[expected_data].write(1);
                write_bytes(
                    &mut instruction_data[expected_data + 1..expected_data + 9],
                    &value.to_le_bytes(),
                );
                expected_data += 9;
            } else {
                instruction_data[expected_data].write(0);
                expected_data += 1;
            }
        }

        invoke(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[InstructionAccount::readonly(self.metadata.address())],
                // SAFETY: `instruction_data` was initialized for `expected_data` bytes.
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, expected_data) },
            },
            &[self.metadata],
        )
    }
}
