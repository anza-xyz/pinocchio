#![no_std]

use {
    crate::instructions::{Assign, CreateAccount, Transfer},
    pinocchio::{
        address::declare_id,
        cpi::Signer,
        sysvars::{rent::Rent, Sysvar},
        AccountView, Address, ProgramResult,
    },
};

pub mod instructions;

declare_id!("11111111111111111111111111111111");

/// Create an account with a minimum balance to be rent-exempt.
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
    account: &mut AccountView,
    space: usize,
    owner: &Address,
    payer: &AccountView,
    rent_sysvar: Option<&AccountView>,
    signers: &[Signer],
) -> ProgramResult {
    let lamports = if let Some(rent_sysvar) = rent_sysvar {
        let rent = Rent::from_account_view(rent_sysvar)?;
        rent.try_minimum_balance(space)?
    } else {
        Rent::get()?.try_minimum_balance(space)?
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

        #[cfg(not(any(feature = "account-resize", feature = "unsafe-account-resize")))]
        // When account resize is not enabled, allocate the required space for the account
        // using the `Allocate` instruction.
        crate::instructions::Allocate {
            account,
            space: space as u64,
        }
        .invoke_signed(signers)?;

        // Assign the account to the specified owner.
        Assign { account, owner }.invoke_signed(signers)?;

        #[cfg(any(feature = "account-resize", feature = "unsafe-account-resize"))]
        // Allocate the required space for the account using the `AccountView::resize`.
        //
        // SAFETY: There are no active borrows of the `account`.
        // This was checked by the `Assign` CPI above.
        unsafe {
            #[cfg(feature = "account-resize")]
            <AccountView as pinocchio::Resize>::resize_unchecked(account, space)?;

            #[cfg(all(feature = "unsafe-account-resize", not(feature = "account-resize")))]
            <AccountView as pinocchio::UnsafeResize>::resize(account, space);
        }

        Ok(())
    }
}
