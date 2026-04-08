mod account;
mod account_state;
mod account_type;
mod mint;
mod multisig;

use solana_program_error::{ProgramError, ProgramResult};
pub use {account::*, account_state::*, account_type::*, mint::*, multisig::*};

/// Extension data is only written after `Account::BASE_LEN`, so extensible
/// accounts store their [`AccountType`] marker immediately after that base
/// region.
///
/// Plain, non-extensible accounts are validated by their exact base length and
/// therefore do not require an account type marker.
const BASE_ACCOUNT_LENGTH: usize = Account::BASE_LEN;

#[inline(always)]
const fn validate_account_type(
    bytes: &[u8],
    account_type: AccountType,
    expected_base_len: usize,
) -> ProgramResult {
    let len = bytes.len();

    if len == expected_base_len {
        return Ok(());
    }

    if len != Multisig::LEN
        && len > BASE_ACCOUNT_LENGTH
        && bytes[BASE_ACCOUNT_LENGTH] == account_type as u8
    {
        return Ok(());
    }

    Err(ProgramError::InvalidAccountData)
}
