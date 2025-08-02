use pinocchio::account_info::AccountInfo;

/// Transfer lamports.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Funding account
///   1. `[WRITE]` Recipient account
pub struct Transfer<'a> {
    /// Funding account.
    pub from: &'a AccountInfo,

    /// Recipient account.
    pub to: &'a AccountInfo,

    /// Amount of lamports to transfer.
    pub lamports: u64,
}

pub const TRANSFER_ACCOUNTS_LEN: usize = 2;
pub const TRANSFER_DATA_SIZE: usize = 12;
