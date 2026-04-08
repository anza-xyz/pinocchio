use crate::state::{Account, Mint};

/// Different kinds of accounts. Note that `Mint`, `Account`, and `Multisig`
/// types are determined exclusively by the size of the account, and are not
/// included in the account data. `AccountType` is only included if extensions
/// have been initialized.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccountType {
    /// Marker for 0 data
    Uninitialized,
    /// Mint account with additional extensions
    Mint,
    /// Token holding account with additional extensions
    Account,
}

impl AccountType {
    /// Return the expected length of an account for the given `AccountType`.
    pub const fn len_for_type(&self) -> usize {
        match self {
            AccountType::Uninitialized => 0,
            AccountType::Mint => Mint::BASE_LEN,
            AccountType::Account => Account::BASE_LEN,
        }
    }
}

impl From<u8> for AccountType {
    fn from(value: u8) -> Self {
        match value {
            0..=2 => unsafe { core::mem::transmute::<u8, AccountType>(value) },
            _ => panic!("invalid account type value: {value}"),
        }
    }
}

impl From<AccountType> for u8 {
    fn from(value: AccountType) -> Self {
        match value {
            AccountType::Uninitialized => 0,
            AccountType::Mint => 1,
            AccountType::Account => 2,
        }
    }
}
