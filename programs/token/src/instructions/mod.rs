mod approve;
mod approve_checked;
mod burn;
mod burn_checked;
mod close_account;
mod freeze_account;
mod initialize_account;
mod initialize_account_2;
mod initialize_account_3;
mod initialize_mint;
mod initialize_mint_2;
mod mint_to;
mod mint_to_checked;
mod revoke;
mod set_authority;
mod sync_native;
mod thaw_account;
mod transfer;
mod transfer_checked;

pub use approve::*;
pub use approve_checked::*;
pub use burn::*;
pub use burn_checked::*;
pub use close_account::*;
pub use freeze_account::*;
pub use initialize_account::*;
pub use initialize_account_2::*;
pub use initialize_account_3::*;
pub use initialize_mint::*;
pub use initialize_mint_2::*;
pub use mint_to::*;
pub use mint_to_checked::*;

pub use revoke::*;
pub use set_authority::*;
pub use sync_native::*;
pub use thaw_account::*;
pub use transfer::*;
pub use transfer_checked::*;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum TokenProgramVariant {
    Legacy,
    Token2022,
}

use pinocchio::pubkey::Pubkey;

use crate::{TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID};
impl From<TokenProgramVariant> for Pubkey {
    fn from(variant: TokenProgramVariant) -> Self {
        match variant {
            TokenProgramVariant::Legacy => TOKEN_PROGRAM_ID,
            TokenProgramVariant::Token2022 => TOKEN_2022_PROGRAM_ID,
        }
    }
}
