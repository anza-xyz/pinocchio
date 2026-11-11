#![no_std]

use {
    crate::instructions::CreateAccountAllowPrefund,
    pinocchio::{address::declare_id, cpi::Signer, AccountView, Address, ProgramResult},
};

pub mod instructions;

declare_id!("11111111111111111111111111111111");

/// Create an account with the minimum balance required for rent exemption.
///
/// Calling this function on an account that is already initialized with
/// the requested size will fail.
#[deprecated(
    since = "0.7.1",
    note = "Use `CreateAccountAllowPrefund::with_minimum_balance` instead"
)]
#[allow(deprecated)]
#[inline(always)]
pub fn create_account_with_minimum_balance(
    account: &mut AccountView,
    space: usize,
    owner: &Address,
    payer: &AccountView,
    rent_sysvar: Option<&AccountView>,
) -> ProgramResult {
    create_account_with_minimum_balance_signed(account, space, owner, payer, rent_sysvar, &[])
}

/// Create an account with the minimum balance required for rent exemption.
///
/// When creating a PDA `account`, the PDA signer seeds must be provided
/// via `signers`.
///
/// The account will be funded by the `payer` if its current lamports
/// are insufficient for rent-exemption. The payer can be a PDA signer
/// owned by the system program and its signer seeds can be provided
/// via `signers`.
///
/// Calling this function on an account that is already initialized with
/// the requested size will fail.
#[deprecated(
    since = "0.7.1",
    note = "Use `CreateAccountAllowPrefund::with_minimum_balance` instead"
)]
#[inline(always)]
pub fn create_account_with_minimum_balance_signed(
    account: &mut AccountView,
    space: usize,
    owner: &Address,
    payer: &AccountView,
    rent_sysvar: Option<&AccountView>,
    signers: &[Signer],
) -> ProgramResult {
    CreateAccountAllowPrefund::with_minimum_balance(
        payer,
        account,
        space as u64,
        owner,
        rent_sysvar,
    )?
    .invoke_signed(signers)
}

#[cfg(any(feature = "account-resize", feature = "unsafe-account-resize"))]
/// Create a program account with the minimum balance required for rent
/// exemption.
///
/// Calling this function on an account that is already initialized with
/// the requested size will fail.
#[deprecated(
    since = "0.7.1",
    note = "Use `CreateAccountAllowPrefund::with_minimum_balance` instead"
)]
#[allow(deprecated)]
#[inline(always)]
pub fn create_program_account_with_minimum_balance(
    account: &mut AccountView,
    space: usize,
    owner: &Address,
    payer: &AccountView,
    rent_sysvar: Option<&AccountView>,
) -> ProgramResult {
    create_account_with_minimum_balance(account, space, owner, payer, rent_sysvar)
}

#[cfg(any(feature = "account-resize", feature = "unsafe-account-resize"))]
/// Create a program account with the minimum balance required for rent
/// exemption.
///
/// The PDA signer seeds must be provided via `signers`.
///
/// The account will be funded by the `payer` if its current lamports
/// are insufficient for rent-exemption. The payer can be a PDA signer
/// owned by the system program and its signer seeds can be provided
/// via the `signers`.
///
/// Calling this function on an account that is already initialized with
/// the requested size will fail.
#[deprecated(
    since = "0.7.1",
    note = "Use `CreateAccountAllowPrefund::with_minimum_balance` instead"
)]
#[allow(deprecated)]
#[inline(always)]
pub fn create_program_account_with_minimum_balance_signed(
    account: &mut AccountView,
    space: usize,
    owner: &Address,
    payer: &AccountView,
    rent_sysvar: Option<&AccountView>,
    signers: &[Signer],
) -> ProgramResult {
    create_account_with_minimum_balance_signed(account, space, owner, payer, rent_sysvar, signers)
}
