extern crate alloc;

use alloc::vec::Vec;
use core::str;
use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    InstructionAccount, InstructionView,
};
use solana_program_error::ProgramResult;

/// Field type for metadata updates
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Field<'a> {
    /// The name field, corresponding to `TokenMetadata.name`
    Name = 0,
    /// The symbol field, corresponding to `TokenMetadata.symbol`
    Symbol = 1,
    /// The uri field, corresponding to `TokenMetadata.uri`
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

    /// Returns the serialized size of the key field if present
    /// Returns 0 for built-in fields (Name, Symbol, Uri)
    pub fn key_size(&self) -> usize {
        match self {
            Field::Key(key) => 4 + key.len(), // 4 bytes for length prefix + key bytes
            _ => 0,
        }
    }
}

/// Initialize token metadata for a Token-2022 mint.
///
/// This instruction creates and populates the metadata account with
/// the token's name, symbol, and URI.
///
/// ### Accounts:
///   0. `[WRITE]` Metadata account
///   1. `[]` Update authority
///   2. `[]` Mint
///   3. `[SIGNER]` Mint authority
pub struct InitializeTokenMetadata<'a, 'b> {
    /// The metadata account to initialize
    pub metadata: &'a AccountView,
    /// The authority that can update the metadata
    pub update_authority: &'a AccountView,
    /// The mint account
    pub mint: &'a AccountView,
    /// The mint authority (must sign)
    pub mint_authority: &'a AccountView,
    /// Token name
    pub name: &'a str,
    /// Token symbol
    pub symbol: &'a str,
    /// URI to token metadata
    pub uri: &'a str,
    /// Token program (Token-2022).
    pub token_program: &'b Address,
}

impl InitializeTokenMetadata<'_, '_> {
    /// Based on spl_token_metadata_interface hash
    pub const DISCRIMINATOR: [u8; 8] = [210, 225, 30, 162, 88, 184, 77, 141];

    /// Invoke the InitializeTokenMetadata instruction
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    /// Invoke the InitializeTokenMetadata instruction with signers
    ///
    /// Instruction data layout:
    /// - [0..8]: instruction discriminator (8 bytes)
    /// - [8..12]: name length (4 bytes, u32)
    /// - [12..12+n1]: name string (n1 bytes, UTF-8)
    /// - [...]: symbol length (4 bytes, u32)
    /// - [...]: symbol string (n2 bytes, UTF-8)
    /// - [...]: uri length (4 bytes, u32)
    /// - [...]: uri string (n3 bytes, UTF-8)
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let ix_len = 8 // instruction discriminator
                + 4 // name length
                + self.name.len()
                + 4 // symbol length
                + self.symbol.len()
                + 4 // uri length
                + self.uri.len();
        let mut ix_data: Vec<u8> = Vec::with_capacity(ix_len);

        ix_data.extend(Self::DISCRIMINATOR);

        // Set name length and name data bytes
        let name_len = self.name.len() as u32;
        ix_data.extend(&name_len.to_le_bytes());
        ix_data.extend(self.name.as_bytes());

        // Set symbol length and symbol data bytes
        let symbol_len = self.symbol.len() as u32;
        ix_data.extend(&symbol_len.to_le_bytes());
        ix_data.extend(self.symbol.as_bytes());

        // Set uri length and uri data bytes
        let uri_len = self.uri.len() as u32;
        ix_data.extend(&uri_len.to_le_bytes());
        ix_data.extend(self.uri.as_bytes());

        let instruction_accounts: [InstructionAccount; 4] = [
            InstructionAccount::writable(self.metadata.address()),
            InstructionAccount::readonly(self.update_authority.address()),
            InstructionAccount::readonly(self.mint.address()),
            InstructionAccount::readonly_signer(self.mint_authority.address()),
        ];

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &instruction_accounts,
            data: &ix_data[..ix_len],
        };

        invoke_signed(
            &instruction,
            &[
                self.metadata,
                self.update_authority,
                self.mint,
                self.mint_authority,
            ],
            signers,
        )
    }
}

/// Remove a key-value pair from token metadata.
///
/// This instruction removes a custom key from the additional metadata.
/// If idempotent is true, the instruction succeeds even if the key doesn't exist.
///
/// ### Accounts:
///   0. `[WRITE]` Metadata account
///   1. `[SIGNER]` Update authority
pub struct RemoveKey<'a, 'b> {
    /// The metadata account to update
    pub metadata: &'a AccountView,
    /// The account authorized to update the metadata
    pub update_authority: &'a AccountView,
    /// The key to remove from the metadata
    pub key: &'a str,
    /// Whether the operation should be idempotent
    pub idempotent: bool,
    /// Token program (Token-2022).
    pub token_program: &'b Address,
}

impl RemoveKey<'_, '_> {
    /// Based on spl_token_metadata_interface hash
    pub const DISCRIMINATOR: [u8; 8] = [234, 18, 32, 56, 89, 141, 37, 181];

    /// Invoke the RemoveKey instruction
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    /// Invoke the RemoveKey instruction with signers
    ///
    /// Instruction data layout:
    /// - [0..8]: instruction discriminator (8 bytes)
    /// - [8..9]: idempotent flag (1 byte, bool as u8)
    /// - [9..13]: key length (4 bytes, u32)
    /// - [13..13+k]: key string (k bytes, UTF-8)
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let ix_len = 8 // instruction discriminator
            + 1 // idempotent flag
            + 4 // key length
            + self.key.len(); // key data

        let mut ix_data: Vec<u8> = Vec::with_capacity(ix_len);

        // Set 8-byte discriminator for RemoveKey
        ix_data.extend(Self::DISCRIMINATOR);

        // Set idempotent flag
        ix_data.push(self.idempotent as u8);

        // Set serialized key data
        let key_len = self.key.len() as u32;
        ix_data.extend(&key_len.to_le_bytes());
        ix_data.extend(self.key.as_bytes());

        // Create account metas
        let account_metas: [InstructionAccount; 2] = [
            InstructionAccount::writable(self.metadata.address()),
            InstructionAccount::readonly_signer(self.update_authority.address()),
        ];

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &account_metas,
            data: &ix_data,
        };

        invoke_signed(
            &instruction,
            &[self.metadata, self.update_authority],
            signers,
        )
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

    /// Invoke the UpdateField instruction
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    /// Invoke the UpdateField instruction with signers
    ///
    /// Instruction data layout for Field::Key:
    /// - [0..8]: instruction discriminator (8 bytes)
    /// - [8..9]: field enum type (1 byte, u8)
    /// - [9..13]: key length (4 bytes, u32) = k
    /// - [13..13+k]: key string (k bytes, UTF-8)
    /// - [13+k..17+k]: value length (4 bytes, u32) = v
    /// - [17+k..17+k+v]: value string (v bytes, UTF-8)
    ///
    /// Instruction data layout for Field::Name/Symbol/Uri:
    /// - [0..8]: instruction discriminator (8 bytes)
    /// - [8..9]: field enum type (1 byte, u8)
    /// - [9..13]: value length (4 bytes, u32) = v
    /// - [13..13+v]: value string (v bytes, UTF-8)
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

        let account_metas: [InstructionAccount; 2] = [
            InstructionAccount::writable(self.metadata.address()),
            InstructionAccount::readonly_signer(self.update_authority.address()),
        ];

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &account_metas,
            data: &ix_data,
        };

        invoke_signed(
            &instruction,
            &[self.metadata, self.update_authority],
            signers,
        )
    }
}
