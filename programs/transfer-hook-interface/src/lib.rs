//! The pinocchio-transfer-hook-interface library provides onchain helpers for
//! resolving the additional accounts required for executing a transfer-hook
//! instruction.

#![no_std]

use solana_address::Address;

extern crate alloc;

pub mod instructions;

/// Namespace for all programs implementing transfer-hook
pub const NAMESPACE: &str = "spl-transfer-hook-interface";

/// Seed for the state
const EXTRA_ACCOUNT_METAS_SEED: &[u8] = b"extra-account-metas";

/// Get the state address PDA
#[inline(always)]
pub fn get_extra_account_metas_address(mint: &Address, program_id: &Address) -> Address {
    get_extra_account_metas_address_and_bump_seed(mint, program_id).0
}

/// Function used by programs implementing the interface, when creating the PDA,
/// to also get the bump seed
#[inline(always)]
pub fn get_extra_account_metas_address_and_bump_seed(
    mint: &Address,
    program_id: &Address,
) -> (Address, u8) {
    Address::derive_program_address(&collect_extra_account_metas_seeds(mint), program_id)
        .unwrap_or_else(|| panic!("Unable to find a viable program address bump seed"))
}

/// Function used by programs implementing the interface, when creating the PDA,
/// to get all of the PDA seeds
#[inline(always)]
pub fn collect_extra_account_metas_seeds(mint: &Address) -> [&[u8]; 2] {
    [EXTRA_ACCOUNT_METAS_SEED, mint.as_ref()]
}

/// Function used by programs implementing the interface, when creating the PDA,
/// to sign for the PDA
#[inline(always)]
pub fn collect_extra_account_metas_signer_seeds<'a>(
    mint: &'a Address,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [EXTRA_ACCOUNT_METAS_SEED, mint.as_ref(), bump_seed]
}
