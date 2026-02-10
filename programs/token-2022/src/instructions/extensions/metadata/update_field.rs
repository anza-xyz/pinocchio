// NOTE: Metadata interface instructions use `Vec` for instruction data because
// the payload contains variable-length strings whose total size is not known at
// compile time.  The rest of the crate uses stack-allocated `UNINIT_BYTE` arrays,
// which is possible only when the maximum data size is bounded and small.
extern crate alloc;

use alloc::vec::Vec;
use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    InstructionAccount, InstructionView,
};
use solana_program_error::ProgramResult;

/// Field type for metadata updates.
///
/// The `#[repr(u8)]` controls the in-memory discriminant only;
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Field<'a> {
    /// The name field, corresponding to `Metadata.name`
    Name = 0,
    /// The symbol field, corresponding to `Metadata.symbol`
    Symbol = 1,
    /// The uri field, corresponding to `Metadata.uri`
    Uri = 2,
    /// A user field, whose key is given by the associated string
    Key(&'a str) = 3,
}

impl Field<'_> {
    pub fn to_u8(&self) -> u8 {
        match self {
            Field::Name => 0,
            Field::Symbol => 1,
            Field::Uri => 2,
            Field::Key(_) => 3,
        }
    }

    /// Returns the serialized size of the key field if present.
    /// Returns 0 for built-in fields (Name, Symbol, Uri).
    pub fn key_size(&self) -> usize {
        match self {
            Field::Key(key) => 4 + key.len(), // 4 bytes for length prefix + key bytes
            _ => 0,
        }
    }
}

/// Update a field in token metadata.
///
/// This instruction updates either a built-in field (Name, Symbol, Uri)
/// or a custom key-value pair in the additional metadata.
///
/// ### Accounts:
///   0. `[WRITE]` Metadata account
///   1. `[SIGNER]` Update authority
pub struct UpdateField<'a, 'b> {
    /// The metadata account to update
    pub metadata: &'a AccountView,
    /// The authority that can sign to update the metadata
    pub update_authority: &'a AccountView,
    /// Field to update in the metadata
    pub field: Field<'a>,
    /// Value to write for the field
    pub value: &'a str,
    /// Token program (Token-2022).
    pub token_program: &'b Address,
}

impl UpdateField<'_, '_> {
    pub const DISCRIMINATOR: [u8; 8] = [221, 233, 49, 45, 181, 202, 220, 200];

    /// Invoke the `UpdateField` instruction.
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    /// Invoke the `UpdateField` instruction with signers.
    ///
    /// Instruction data layout for `Field::Key`:
    /// - `[0..8]`: instruction discriminator (8 bytes)
    /// - `[8..9]`: field enum type (1 byte, `u8`)
    /// - `[9..13]`: key length (4 bytes, `u32`)
    /// - `[13..13+K]`: key string (K bytes, UTF-8)
    /// - `[..+4]`: value length (4 bytes, `u32`)
    /// - `[..+V]`: value string (V bytes, UTF-8)
    ///
    /// Instruction data layout for `Field::Name`/`Symbol`/`Uri`:
    /// - `[0..8]`: instruction discriminator (8 bytes)
    /// - `[8..9]`: field enum type (1 byte, `u8`)
    /// - `[9..13]`: value length (4 bytes, `u32`)
    /// - `[13..13+V]`: value string (V bytes, UTF-8)
    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let ix_len = 8 // instruction discriminator
            + 1 // field type
            + self.field.key_size()
            + 4 // value length
            + self.value.len();

        let mut ix_data: Vec<u8> = Vec::with_capacity(ix_len);

        // Set 8-byte discriminator for UpdateField
        ix_data.extend(Self::DISCRIMINATOR);
        ix_data.push(self.field.to_u8());

        // Set serialized key data in buffer if Field is Key type
        if let Field::Key(key) = self.field {
            let key_len = key.len() as u32;
            ix_data.extend(key_len.to_le_bytes());
            ix_data.extend(key.as_bytes());
        }

        // Set serialized value data in buffer
        let value_len = self.value.len() as u32;
        ix_data.extend(value_len.to_le_bytes());
        ix_data.extend(self.value.as_bytes());

        let instruction_accounts: [InstructionAccount; 2] = [
            InstructionAccount::writable(self.metadata.address()),
            InstructionAccount::readonly_signer(self.update_authority.address()),
        ];

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &instruction_accounts,
            data: &ix_data,
        };

        invoke_signed(
            &instruction,
            &[self.metadata, self.update_authority],
            signers,
        )
    }
}
