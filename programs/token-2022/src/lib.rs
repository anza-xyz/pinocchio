#![no_std]

pub mod instructions;
pub mod state;

use {
    core::mem::MaybeUninit,
    pinocchio_token::TokenProgram,
    solana_address::Address,
    solana_program_error::{ProgramError, ProgramResult},
};

solana_address::declare_id!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

/// Struct to represent the SPL Token-2022 program.
///
/// This struct implements the `TokenProgram` trait, which statically provides
/// the SPL Token-2022 address for instruction building.
pub struct Program2022;

impl TokenProgram for Program2022 {
    const ID: Address = crate::ID;

    #[inline(always)]
    fn verify(address: &Address) -> ProgramResult {
        if address != &Self::ID && address != &pinocchio_token::ID {
            return Err(incorrect_program_id());
        }

        Ok(())
    }
}

/// Cold helper for constructing `ProgramError::IncorrectProgramId` outside the
/// hot path.
#[doc(hidden)]
#[cold]
fn incorrect_program_id() -> ProgramError {
    ProgramError::IncorrectProgramId
}

#[inline(always)]
fn write_bytes(destination: &mut [MaybeUninit<u8>], source: &[u8]) {
    let len = destination.len().min(source.len());
    // SAFETY:
    // - Both pointers have alignment 1.
    // - For valid (non-UB) references, the borrow checker guarantees no overlap.
    // - `len` is bounded by both slice lengths.
    unsafe {
        core::ptr::copy_nonoverlapping(source.as_ptr(), destination.as_mut_ptr() as *mut u8, len);
    }
}

/// The Mint that represents the native token
pub mod native_mint {
    /// There are `10^9` lamports in one SOL.
    pub const DECIMALS: u8 = 9;

    // The Mint for native SOL Token accounts.
    solana_address::declare_id!("9pan9bMn5HatX4EJdBwg9VgCa7Uz5HL8N1m5D3NdXejP");

    /// Seed for the native mint's program-derived address
    pub const PROGRAM_ADDRESS_SEEDS: &[&[u8]] = &["native-mint".as_bytes(), &[255]];

    #[cfg(test)]
    mod tests {
        use {super::*, solana_address::Address};

        #[test]
        fn expected_native_mint_id() {
            let native_mint_id =
                Address::create_program_address(PROGRAM_ADDRESS_SEEDS, &crate::id()).unwrap();
            assert_eq!(id(), native_mint_id);
        }
    }
}
