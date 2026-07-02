#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;
use solana_address::Address;

pub mod instructions;
pub mod state;

solana_address::declare_id!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

/// A trait for token programs that can be used in a CPI with a statically known
/// program address.
pub trait TokenProgram {
    const ID: Address;
}

/// Struct to represent the SPL Token program.
///
/// This struct implements the `TokenProgram` trait, which statically provides
/// the SPL Token address for instruction building.
pub struct Program;

impl TokenProgram for Program {
    const ID: Address = crate::ID;
}
