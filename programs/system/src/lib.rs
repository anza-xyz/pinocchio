#![no_std]

use pinocchio::{
    account_info::AccountInfo,
    instruction::Signer,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

use crate::instructions::{Assign, CreateAccount, Transfer};

pub mod instructions;

pinocchio_pubkey::declare_id!("11111111111111111111111111111111");

/// Create an account with a minimal balance to be rent-exempt.
///
/// The account will be funded by the `payer` if its current lamports
/// are insufficient for rent-exemption.
#[inline(always)]
pub fn create_account_with_minimal_balance(
    account: &AccountInfo,
    space: usize,
    owner: &Pubkey,
    payer: &AccountInfo,
    signers: &[Signer],
    rent_sysvar: Option<&AccountInfo>,
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
            .invoke()?;
        }

        // Assign the account to the specified owner.
        Assign { account, owner }.invoke_signed(signers)?;

        // Allocate the required space for the account.
        //
        // SAFETY: There are no active borrows of the `account`.
        // This was checked by the `Assign` CPI above.
        unsafe { account.resize_unchecked(space)? };

        Ok(())
    }
}
