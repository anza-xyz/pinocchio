use pinocchio::pubkey::Pubkey;

use super::{BaseState, Extension, ExtensionType};

#[derive(Debug, Clone, Copy, PartialEq)]
/// Metadata for a token
pub struct TokenMetadata<'s> {
    /// The authority that can sign to update the metadata
    pub update_authority: Pubkey,
    /// The associated mint, used to counter spoofing to be sure that metadata
    /// belongs to a particular mint
    pub mint: Pubkey,
    /// The longer name of the token
    pub name: &'s str,
    /// The shortened symbol for the token
    pub symbol: &'s str,
    /// The URI pointing to richer metadata
    pub uri: &'s str,
    /// Any additional metadata about the token as key-value pairs. The program
    /// must avoid storing the same key twice.
    pub additional_metadata: &'s [(&'s str, &'s str)],
}

impl<'t> TokenMetadata<'t> {
    /// The length of the `TokenMetadata` account data.
    pub const LEN: usize = core::mem::size_of::<TokenMetadata>();
}

impl Extension for TokenMetadata<'_> {
    const TYPE: ExtensionType = ExtensionType::TokenMetadata;
    const LEN: usize = Self::LEN;
    const BASE_STATE: BaseState = BaseState::Mint;
}
