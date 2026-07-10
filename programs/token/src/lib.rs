#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;
use {
    solana_address::Address,
    solana_program_error::{ProgramError, ProgramResult},
};

pub mod instructions;
pub mod state;

solana_address::declare_id!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

/// A trait for token programs that can be used in a CPI with a statically known
/// program address.
pub trait TokenProgram {
    const ID: Address;

    /// Returns `Ok(())` when `address` is accepted for cross-program
    /// invocations.
    ///
    /// Instructions may accept addresses other than `Self::ID` when a
    /// compatible program can process the same instruction layout.
    #[inline(always)]
    fn verify(address: &Address) -> ProgramResult {
        if address != &Self::ID {
            return Err(incorrect_program_id());
        }

        Ok(())
    }
}

/// Struct to represent the SPL Token program.
///
/// This struct implements the `TokenProgram` trait, which statically provides
/// the SPL Token address for instruction building.
pub struct Program;

impl TokenProgram for Program {
    const ID: Address = crate::ID;
}

/// Cold helper for constructing `ProgramError::IncorrectProgramId` outside the
/// hot path.
#[doc(hidden)]
#[cold]
fn incorrect_program_id() -> ProgramError {
    ProgramError::IncorrectProgramId
}
