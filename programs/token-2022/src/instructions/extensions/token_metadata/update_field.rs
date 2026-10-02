use {
    crate::{write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// A field of the `TokenMetadata` to update.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field<'a> {
    /// The `name` field.
    Name,

    /// The `symbol` field.
    Symbol,

    /// The `uri` field.
    Uri,

    /// A user field, whose key is given by the associated string.
    Key(&'a str),
}

/// Updates a field in a `TokenMetadata` entry. If the field is a `Key`
/// that does not exist yet, it is added.
///
/// The metadata account must have enough lamports to cover the rent of any
/// additional space.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The metadata account.
///   1. `[signer]` The update authority.
pub struct UpdateField<'a, 'b> {
    /// The metadata account.
    pub metadata: &'a AccountView,

    /// The update authority.
    pub update_authority: &'a AccountView,

    /// The field to update.
    pub field: Field<'b>,

    /// The value of the field.
    pub value: &'b str,

    /// The token program.
    pub token_program: &'b Address,
}

impl UpdateField<'_, '_> {
    /// Hash of `spl_token_metadata_interface:updating_field`.
    pub const DISCRIMINATOR: [u8; 8] = [221, 233, 49, 45, 181, 202, 220, 200];

    /// Maximum instruction data length.
    pub const MAX_DATA_LEN: usize = 1024;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data.

        let key = match self.field {
            Field::Key(key) => Some(key),
            _ => None,
        };

        let expected_data = 8 + 1 + key.map_or(0, |key| 4 + key.len()) + 4 + self.value.len();

        if expected_data > Self::MAX_DATA_LEN {
            return Err(ProgramError::InvalidArgument);
        }

        let mut instruction_data = [UNINIT_BYTE; Self::MAX_DATA_LEN];

        write_bytes(&mut instruction_data[0..8], &Self::DISCRIMINATOR);

        instruction_data[8].write(match self.field {
            Field::Name => 0,
            Field::Symbol => 1,
            Field::Uri => 2,
            Field::Key(_) => 3,
        });

        let mut offset = 9;

        for value in key.into_iter().chain([self.value]) {
            write_bytes(
                &mut instruction_data[offset..offset + 4],
                &(value.len() as u32).to_le_bytes(),
            );
            write_bytes(
                &mut instruction_data[offset + 4..offset + 4 + value.len()],
                value.as_bytes(),
            );
            offset += 4 + value.len();
        }

        invoke_signed(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[
                    InstructionAccount::writable(self.metadata.address()),
                    InstructionAccount::readonly_signer(self.update_authority.address()),
                ],
                // SAFETY: instruction data is initialized up to `expected_data`.
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, expected_data) },
            },
            &[self.metadata, self.update_authority],
            signers,
        )
    }
}
