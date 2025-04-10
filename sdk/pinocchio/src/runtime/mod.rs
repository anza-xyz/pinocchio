#[cfg(all(not(target_os = "solana"), not(feature = "test")))]
use blackbox::BlackBoxRuntime;
#[cfg(all(not(target_os = "solana"), feature = "test"))]
use mock::MockRuntime;
#[cfg(target_os = "solana")]
use solana::SolanaRuntime;

use crate::{
    account_info::AccountInfo,
    instruction::{Instruction, Signer},
    msg, pubkey, ProgramResult,
};

mod blackbox;
pub mod mock;
mod solana;

#[cfg(target_os = "solana")]
pub type TargetRuntime = SolanaRuntime;

#[cfg(all(not(target_os = "solana"), not(feature = "test")))]
pub type TargetRuntime = BlackBoxRuntime;

#[cfg(all(not(target_os = "solana"), feature = "test"))]
pub type TargetRuntime = MockRuntime;

pub trait Runtime {
    ////////////////////////////////////////////////////////////////////////////
    // LOG SYS CALLS
    ////////////////////////////////////////////////////////////////////////////

    /// Print a string to the log.
    fn sol_log(message: &str);

    /// Print 64-bit values represented as hexadecimal to the log.
    fn sol_log_64(arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64);

    /// Print some slices as base64.
    fn sol_log_data(data: &[&[u8]]);

    /// Print the hexadecimal representation of a slice.
    fn sol_log_slice(slice: &[u8]) {
        for (i, s) in slice.iter().enumerate() {
            Self::sol_log_64(0, 0, 0, i as u64, *s as u64);
        }
    }

    /// Print the hexadecimal representation of the program's input parameters.
    ///
    /// - `accounts` - A slice of [`AccountInfo`].
    /// - `data` - The instruction data.
    fn sol_log_params(accounts: &[AccountInfo], data: &[u8]) {
        for (i, account) in accounts.iter().enumerate() {
            msg!("AccountInfo");
            Self::sol_log_64(0, 0, 0, 0, i as u64);
            msg!("- Is signer");
            Self::sol_log_64(0, 0, 0, 0, account.is_signer() as u64);
            msg!("- Key");
            pubkey::log(account.key());
            msg!("- Lamports");
            Self::sol_log_64(0, 0, 0, 0, account.lamports());
            msg!("- Account data length");
            Self::sol_log_64(0, 0, 0, 0, account.data_len() as u64);
            msg!("- Owner");
            // SAFETY: The `owner` reference is only used for logging.
            pubkey::log(unsafe { account.owner() });
        }
        msg!("Instruction data");
        Self::sol_log_slice(data);
    }

    /// Print the remaining compute units available to the program.
    fn sol_log_compute_units();

    ////////////////////////////////////////////////////////////////////////////
    // MEM SYS CALLS
    ////////////////////////////////////////////////////////////////////////////

    unsafe fn sol_memcpy(dst: &mut [u8], src: &[u8], n: usize);

    unsafe fn sol_memmove(dst: *mut u8, src: *mut u8, n: usize);

    unsafe fn sol_memcmp(s1: &[u8], s2: &[u8], n: usize) -> i32;

    unsafe fn sol_memset(s: &mut [u8], c: u8, n: usize);

    ////////////////////////////////////////////////////////////////////////////
    // CPI CALLS
    ////////////////////////////////////////////////////////////////////////////

    /// Invoke a cross-program instruction with signatures.
    ///
    /// # Important
    ///
    /// The accounts on the `account_infos` slice must be in the same order as the
    /// `accounts` field of the `instruction`.
    fn invoke_signed<const ACCOUNTS: usize>(
        instruction: &Instruction,
        account_infos: &[&AccountInfo; ACCOUNTS],
        signers_seeds: &[Signer],
    ) -> ProgramResult;

    /// Invoke a cross-program instruction with signatures.
    ///
    /// # Important
    ///
    /// The accounts on the `account_infos` slice must be in the same order as the
    /// `accounts` field of the `instruction`.
    ///
    /// # Safety
    ///
    /// If any of the writable accounts passed to the callee contain data that is
    /// borrowed within the calling program, and that data is written to by the
    /// callee, then Rust's aliasing rules will be violated and cause undefined
    /// behavior.
    unsafe fn invoke_signed_access_unchecked<const ACCOUNTS: usize>(
        instruction: &Instruction,
        account_infos: &[&AccountInfo; ACCOUNTS],
        signers_seeds: &[Signer],
    ) -> ProgramResult;
}
