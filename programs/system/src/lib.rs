#![no_std]

use pinocchio::{
    account::AccountView,
    address::declare_id,
    instruction::Signer,
    sysvars::{rent::Rent, Sysvar},
    Address, ProgramResult,
};

use crate::instructions::{Assign, CreateAccount, Transfer};

pub mod instructions;

declare_id!("11111111111111111111111111111111");

/// Create an account with a minimum balance to be rent-exempt.
#[inline(always)]
pub fn create_account_with_minimum_balance(
    account: &AccountView,
    space: usize,
    owner: &Address,
    payer: &AccountView,
    rent_sysvar: Option<&AccountView>,
) -> ProgramResult {
    create_account_with_minimum_balance_signed(account, space, owner, payer, rent_sysvar, &[])
}

/// Create an account with a minimum balance to be rent-exempt.
///
/// When creating a PDA `account`, the PDA signer seeds must be provided
/// via the `signers`.
///
/// The account will be funded by the `payer` if its current lamports
/// are insufficient for rent-exemption. The payer can be a PDA signer
/// owned by the system program and its signer seeds can be provided
/// via the `signers`.
#[inline(always)]
pub fn create_account_with_minimum_balance_signed(
    account: &AccountView,
    space: usize,
    owner: &Address,
    payer: &AccountView,
    rent_sysvar: Option<&AccountView>,
    signers: &[Signer],
) -> ProgramResult {
    let lamports = if let Some(rent_sysvar) = rent_sysvar {
        let rent = Rent::from_account_info(rent_sysvar)?;
        rent.minimum_balance(space)
    } else {
        Rent::get()?.minimum_balance(space)
    };

    if account.lamports() == 0 {
        // Create the account if it does not exist.
        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: space as u64,
            owner,
        }
        .invoke_signed(signers)
    } else {
        let required_lamports = lamports.saturating_sub(account.lamports());

        // Transfer lamports from `payer` to `account` if needed.
        if required_lamports > 0 {
            Transfer {
                from: payer,
                to: account,
                lamports: required_lamports,
            }
            .invoke_signed(signers)?;
        }

        // Assign the account to the specified owner.
        Assign { account, owner }.invoke_signed(signers)?;

        // Allocate the required space for the account.
        //
        // SAFETY: There are no active borrows of the `account`.
        // This was checked by the `Assign` CPI above.
        unsafe { account.resize_unchecked(space) }
    }
}
