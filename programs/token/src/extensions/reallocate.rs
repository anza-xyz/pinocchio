// Instruction

use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

pub struct Reallocate<'a, const EXTENSIONS: usize> {
    /// The token account to reallocate.
    pub token_account: &'a AccountInfo,
    /// The payer for the reallocation.
    pub payer: &'a AccountInfo,
    /// The system program account for reallocation.
    pub system_program: &'a AccountInfo,
    /// The token account authority.
    pub authority: &'a AccountInfo,
    /// array of extension types
    pub extension_types: &'a [u8; EXTENSIONS],
}
