#![no_std]

pub mod instructions;
pub mod state;

use solana_address::{declare_id, Address};

declare_id!("Feature111111111111111111111111111111111111");

/// The incinerator account — lamports credited to this address are burned
/// at the end of the current block.
pub const INCINERATOR_ID: Address =
    Address::from_str_const("1nc1nerator11111111111111111111111111111111");
