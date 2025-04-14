use core::str;

use super::{get_extension_data_bytes_for_variable_pack, BaseState, Extension, ExtensionType};
use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};
use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

/// State for Metadata for a token
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct TokenMetadata<'a> {
    /// The authority that can sign to update the metadata
    pub update_authority: Pubkey,
    /// The associated mint, used to counter spoofing to be sure that metadata
    /// belongs to a particular mint
    pub mint: Pubkey,
    /// The length of the name
    pub name_len: u32,
    /// The longer name of the token
    pub name: &'a str,
    /// The length of the symbol
    pub symbol_len: u32,
    /// The shortened symbol for the token
    pub symbol: &'a str,
    /// The length of the URI
    pub uri_len: u32,
    /// The URI pointing to richer metadata
    pub uri: &'a str,
    /// The length of the additional metadata
    pub additional_metadata_len: u32,
    /// The additional metadata about the token as key-value pairs. The program
    /// must avoid storing the same key twice.
    pub additional_metadata: &'a [u8],
}

impl TokenMetadata<'_> {
    /// The fixed size of the metadata account: 80 bytes
    /// [32 (update_authority) + 32 (mint) + 4 (size of name ) + 4 (size of symbol) + 4 (size of uri) + 4 (size of additional_metadata)]
    pub const SIZE_METADATA_LEN: usize = 80;

    /// Return a `TokenMetadata` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info<'a>(
        account_info: AccountInfo,
    ) -> Result<TokenMetadata<'a>, ProgramError> {
        if account_info.data_len() < Self::SIZE_METADATA_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let account_data = account_info.try_borrow_data()?;

        let metadata_bytes =
            get_extension_data_bytes_for_variable_pack::<TokenMetadata>(account_data.as_ref())
                .ok_or(ProgramError::InvalidAccountData)?;

        Self::from_bytes(metadata_bytes)
    }

    pub(crate) fn from_bytes<'a>(data: &[u8]) -> Result<TokenMetadata<'a>, ProgramError> {
        if data.len() < Self::SIZE_METADATA_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        let mut offset: usize = 0;

        let update_authority = unsafe { &*(data.as_ptr() as *const [u8; 32]) };

        offset += 32;

        let mint = unsafe { &*(data.as_ptr().add(offset) as *const [u8; 32]) };

        offset += 32;

        let name_len =
            &u32::from_le_bytes(unsafe { *(data.as_ptr().add(offset) as *const [u8; 4]) });

        offset += 4;

        let name = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                data.as_ptr().add(offset),
                *name_len as usize,
            ))
        };

        offset += *name_len as usize;

        let symbol_len =
            &u32::from_le_bytes(unsafe { *(data.as_ptr().add(offset) as *const [u8; 4]) });

        offset += 4;

        let symbol = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                data.as_ptr().add(offset),
                *symbol_len as usize,
            ))
        };

        offset += *symbol_len as usize;

        let uri_len =
            &u32::from_le_bytes(unsafe { *(data.as_ptr().add(offset) as *const [u8; 4]) });

        offset += 4;

        let uri = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                data.as_ptr().add(offset),
                *uri_len as usize,
            ))
        };

        offset += *uri_len as usize;

        let additional_metadata_len =
            &u32::from_le_bytes(unsafe { *(data.as_ptr().add(offset) as *const [u8; 4]) });

        offset += 4;

        let additional_metadata =
            unsafe { core::slice::from_raw_parts(data.as_ptr().add(offset), data.len() - offset) };

        Ok(TokenMetadata {
            update_authority: *update_authority,
            mint: *mint,
            name_len: *name_len,
            name,
            symbol_len: *symbol_len,
            symbol,
            uri_len: *uri_len,
            uri,
            additional_metadata_len: *additional_metadata_len,
            additional_metadata,
        })
    }
}

impl Extension for TokenMetadata<'_> {
    const TYPE: ExtensionType = ExtensionType::TokenMetadata;
    const LEN: usize = Self::SIZE_METADATA_LEN;
    const BASE_STATE: BaseState = BaseState::Mint;
}

// Instructions
/// Instruction to initialize a token metadata account,
/// this takes a constant buffer size to avoid heap allocations
/// `BUF_SIZE` is the size of the buffer to use for the instruction data
/// `BUF_SIZE` = 8 + 4 + name_len + 4 + symbol_len + 4 + uri_len
/// if `BUF_SIZE` is not equal to size of instruction data, it will return an error
pub struct InitializeTokenMetadata<'a, const BUF_SIZE: usize> {
    /// The mint that this metadata pointer is associated with
    pub metadata: &'a AccountInfo,
    /// The authority that can sign to update the metadata
    pub update_authority: &'a AccountInfo,
    /// The associated mint, used to counter spoofing to be sure that metadata
    /// belongs to a particular mint
    pub mint: &'a AccountInfo,
    /// The account address that can update the mint
    pub mint_authority: &'a AccountInfo,
    /// The longer name of the token
    pub name: &'a str,
    /// The shortened symbol for the token
    pub symbol: &'a str,
    /// The URI pointing to richer metadata
    pub uri: &'a str,
}

impl<const BUF_SIZE: usize> InitializeTokenMetadata<'_, BUF_SIZE> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data layout:
        // -  [0..8] : instruction discriminator
        // -  [8..12] : name length (x1)
        // -  [12..12+x1] : name string
        // -  [12+x1..16+x1] : symbol length (x2)
        // -  [16+x1..16+x1+x2]: symbol string
        // -  [16+x1+x2..20+x1+x2] : uri length (x3)
        // -  [20+x1+x2..20+x1+x2+x3] : uri string

        let calculated_ix_size =
            8 + 4 + self.name.len() + 4 + self.symbol.len() + 4 + self.uri.len();

        if calculated_ix_size != BUF_SIZE {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut ix_data = [UNINIT_BYTE; BUF_SIZE];
        let mut offset: usize = 0;

        // Set 8-byte discriminator.
        let discriminator: [u8; 8] = [210, 225, 30, 162, 88, 184, 77, 141];
        write_bytes(&mut ix_data[offset..offset + 8], &discriminator);
        offset += 8;

        // Set name length, and name data bytes,
        let name_bytes = self.name.as_bytes();
        let name_len = name_bytes.len() as u32;
        write_bytes(&mut ix_data[offset..offset + 4], &name_len.to_le_bytes());
        offset += 4;
        write_bytes(&mut ix_data[offset..offset + name_bytes.len()], name_bytes);
        offset += name_bytes.len();

        // Set symbol length, and symbol data bytes,
        let symbol_bytes = self.symbol.as_bytes();
        let symbol_len = symbol_bytes.len() as u32;
        write_bytes(&mut ix_data[offset..offset + 4], &symbol_len.to_le_bytes());
        offset += 4;
        write_bytes(
            &mut ix_data[offset..offset + symbol_bytes.len()],
            symbol_bytes,
        );
        offset += symbol_bytes.len();

        // Set uri length, and uri data bytes,
        let uri_bytes = self.uri.as_bytes();
        let uri_len = uri_bytes.len() as u32;
        write_bytes(&mut ix_data[offset..offset + 4], &uri_len.to_le_bytes());
        offset += 4;
        write_bytes(&mut ix_data[offset..offset + uri_bytes.len()], uri_bytes);
        offset += uri_bytes.len();

        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.metadata.key()),
            AccountMeta::readonly(self.update_authority.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly_signer(self.mint_authority.key()),
        ];

        // Prepare instruction with sliced buffer as data.
        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(ix_data.as_ptr() as _, offset) },
        };

        invoke_signed(&instruction, &[self.metadata, self.mint_authority], signers)
    }
}

#[repr(u8)]
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
}

pub struct UpdateField<'a, const BUF_SIZE: usize> {
    /// The mint that this metadata pointer is associated with
    pub metadata: &'a AccountInfo,
    /// The authority that can sign to update the metadata
    pub update_authority: &'a AccountInfo,
    /// Field to update in the metadata
    pub field: Field<'a>,
    /// Value to write for the field
    pub value: &'a str,
}

impl<const BUF_SIZE: usize> UpdateField<'_, BUF_SIZE> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data layout (if Field type is Key):
        // -  [0..8] [u8]: instruction discriminator
        // -  [8..9] u8: field enum type
        // -  [9..13] u32: key length (x1)
        // -  [13..13+x1] [u8]: key string
        // -  [13+x1..17+x1] u32: value length (x2)
        // -  [17+x1..17+x1+x2] [u8]: value string
        // Instruction data layout (if Field type is not Key):
        // -  [0..8] [u8]: instruction discriminator
        // -  [8..9] u8: field enum type
        // -  [9..13] u32: value length (x1)
        // -  [13..13+x1] [u8]: value string

        let mut ix_data = [UNINIT_BYTE; BUF_SIZE];
        let mut offset: usize = 0;
        // Set 8-byte discriminator.
        let discriminator: [u8; 8] = [221, 233, 49, 45, 181, 202, 220, 200];
        write_bytes(&mut ix_data[offset..offset + 8], &discriminator);
        offset += 8;

        write_bytes(&mut ix_data[offset..offset + 1], &[self.field.to_u8()]);
        offset += 1;

        // Set serialized key data in buffer if Field is Key type.
        if let Field::Key(key) = self.field {
            let key_bytes = key.as_bytes();
            let key_len = key_bytes.len() as u32;
            write_bytes(&mut ix_data[offset..offset + 4], &key_len.to_le_bytes());
            offset += 4;
            write_bytes(&mut ix_data[offset..offset + key_bytes.len()], key_bytes);
            offset += key_bytes.len();
        }

        // Set serialized value data in buffer
        let value_bytes = self.value.as_bytes();
        let value_len = value_bytes.len() as u32;
        write_bytes(&mut ix_data[offset..offset + 4], &value_len.to_le_bytes());
        offset += 4;
        write_bytes(
            &mut ix_data[offset..offset + value_bytes.len()],
            value_bytes,
        );
        offset += value_bytes.len();

        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.metadata.key()),
            AccountMeta::readonly_signer(self.update_authority.key()),
        ];

        // Prepare instruction with sliced buffer as data.
        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(ix_data.as_ptr() as _, offset) },
        };

        invoke_signed(
            &instruction,
            &[self.metadata, self.update_authority],
            signers,
        )
    }
}
