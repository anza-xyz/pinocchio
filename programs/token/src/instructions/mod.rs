mod amount_to_ui_amount;
mod approve;
mod approve_checked;
#[cfg(feature = "batch")]
mod batch;
mod burn;
mod burn_checked;
mod close_account;
mod freeze_account;
mod get_account_data_size;
mod initialize_account;
mod initialize_account_2;
mod initialize_account_3;
mod initialize_immutable_owner;
mod initialize_mint;
mod initialize_mint_2;
mod initialize_multisig;
mod initialize_multisig_2;
mod mint_to;
mod mint_to_checked;
mod revoke;
mod set_authority;
mod sync_native;
mod thaw_account;
mod transfer;
mod transfer_checked;
mod ui_amount_to_amount;

#[cfg(feature = "batch")]
pub use batch::{Batch, Batchable};
pub use {
    amount_to_ui_amount::*, approve::*, approve_checked::*, burn::*, burn_checked::*,
    close_account::*, freeze_account::*, get_account_data_size::*, initialize_account::*,
    initialize_account_2::*, initialize_account_3::*, initialize_immutable_owner::*,
    initialize_mint::*, initialize_mint_2::*, initialize_multisig::*, initialize_multisig_2::*,
    mint_to::*, mint_to_checked::*, revoke::*, set_authority::*, sync_native::*, thaw_account::*,
    transfer::*, transfer_checked::*, ui_amount_to_amount::*,
};
use {
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_instruction_view::{cpi::CpiAccount, InstructionAccount},
    solana_program_error::ProgramError,
};

#[cold]
fn account_borrow_failed_error() -> ProgramError {
    ProgramError::AccountBorrowFailed
}

#[cold]
fn invalid_argument_error() -> ProgramError {
    ProgramError::InvalidArgument
}

#[inline(always)]
fn writable_cpi_account<'cpi>(account: &AccountView) -> Result<CpiAccount<'cpi>, ProgramError> {
    if account.as_ref().is_borrowed() {
        return Err(account_borrow_failed_error());
    }
    Ok(CpiAccount::from(account))
}

#[inline(always)]
fn cpi_account<'cpi>(account: &AccountView) -> Result<CpiAccount<'cpi>, ProgramError> {
    if account.as_ref().is_borrowed_mut() {
        return Err(account_borrow_failed_error());
    }
    Ok(CpiAccount::from(account))
}

/// A trait for instructions that can be used in a CPI context.
pub trait CpiWriter {
    /// Writes the `AccountView`s required by this instruction into the provided
    /// slice.
    ///
    /// Returns the number of accounts written.
    fn write_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        'source: 'cpi;

    /// Writes the `InstructionAccount`s required by this instruction into the
    /// provided slice.
    ///
    /// Returns the number of accounts written.
    fn write_instruction_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        'source: 'cpi;

    /// Writes the instruction data for this instruction into the provided
    /// slice.
    ///
    /// Returns the number of bytes written.
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError>;
}

impl<T: CpiWriter + ?Sized> CpiWriter for &T {
    #[inline(always)]
    fn write_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        'source: 'cpi,
    {
        (**self).write_accounts(accounts)
    }

    #[inline(always)]
    fn write_instruction_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        'source: 'cpi,
    {
        (**self).write_instruction_accounts(accounts)
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        (**self).write_instruction_data(data)
    }
}
