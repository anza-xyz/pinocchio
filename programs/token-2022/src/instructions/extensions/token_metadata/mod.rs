pub mod emit;
pub mod initialize;
pub mod remove_key;
pub mod update_authority;
pub mod update_field;

use crate::Token2022Program;
pub use update_field::Field;

/// Emits the `TokenMetadata` entry as return data.
///
/// The emitted bytes follow the `TokenMetadata` layout, sliced by the
/// optional `start` and `end` offsets. Return data can be fetched using
/// `sol_get_return_data`.
///
/// Accounts expected by this instruction:
///
///   0. `[]` The metadata account.
pub type Emit<'account> = emit::Emit<'account, Token2022Program>;

/// Initializes a `TokenMetadata` TLV entry.
///
/// By the end of the instruction, the metadata account must contain a
/// `TokenMetadata` entry with the given name, symbol and URI.
///
/// Token-2022 only supports storing the metadata in the mint itself, so the
/// metadata account must be the mint. The mint must have the
/// `MetadataPointer` extension pointing to itself and enough lamports to
/// cover the rent of the additional space.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The metadata account.
///   1. `[]` The update authority.
///   2. `[]` The mint.
///   3. `[signer]` The mint authority.
pub type Initialize<'account, 'data> = initialize::Initialize<'account, 'data, Token2022Program>;

/// Removes a key-value pair from the additional metadata of a
/// `TokenMetadata` entry.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The metadata account.
///   1. `[signer]` The update authority.
pub type RemoveKey<'account, 'data> = remove_key::RemoveKey<'account, 'data, Token2022Program>;

/// Updates the update authority of a `TokenMetadata` entry.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The metadata account.
///   1. `[signer]` The current update authority.
pub type UpdateAuthority<'account, 'address> =
    update_authority::UpdateAuthority<'account, 'address, Token2022Program>;

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
pub type UpdateField<'account, 'data> =
    update_field::UpdateField<'account, 'data, Token2022Program>;
