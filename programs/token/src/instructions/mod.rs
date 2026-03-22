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
pub use batch::*;
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
    solana_instruction_view::{cpi::CpiAccount, InstructionAccount},
};

/// Maximum CPI instruction data size. 10 KiB was chosen to ensure that CPI
/// instructions are not more limited than transaction instructions if the size
/// of transactions is doubled in the future.
const MAX_CPI_INSTRUCTION_DATA_LEN: usize = 10 * 1024;

/// A trait for instructions that can be included in a batch instruction.
pub trait Batchable: sealed::Sealed {
    /// Writes the `AccountView`s required by this instruction into the provided slice.
    ///
    /// Returns the number of accounts written.
    fn write_accounts(&self, accounts: &mut [MaybeUninit<CpiAccount>]) -> usize;

    /// Writes the `InstructionAccount`s required by this instruction into the provided slice.
    ///
    /// Returns the number of accounts written.
    fn write_instruction_accounts(&self, accounts: &mut [MaybeUninit<InstructionAccount>])
        -> usize;

    /// Writes the instruction data for this instruction into the provided slice.
    ///
    /// Returns the number of bytes written.
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> usize;
}

/// Implement `Batchable` for references to types that implement `Batchable`.
///
/// This allows the use of `&dyn Batchable` trait object, allowing the
/// use of array of batchable instructions.
impl<T: Batchable + ?Sized> Batchable for &T {
    #[inline(always)]
    fn write_accounts(&self, accounts: &mut [MaybeUninit<CpiAccount>]) -> usize {
        (**self).write_accounts(accounts)
    }

    #[inline(always)]
    fn write_instruction_accounts(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount>],
    ) -> usize {
        (**self).write_instruction_accounts(accounts)
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> usize {
        (**self).write_instruction_data(data)
    }
}

/// Private module to "hide" the `Sealed` trait.
mod sealed {
    /// A sealed trait that prevents external implementations of the
    /// `Batchable` trait.
    pub trait Sealed {}
}
