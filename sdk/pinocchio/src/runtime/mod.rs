#[cfg(not(target_os = "solana"))]
use mock::MockRuntime;
#[cfg(target_os = "solana")]
use solana::SolanaRuntime;

use crate::{account_info::AccountInfo, msg, pubkey};

pub mod mock;
pub mod solana;

#[cfg(target_os = "solana")]
pub type TargetRuntime = SolanaRuntime;

#[cfg(not(target_os = "solana"))]
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
}
