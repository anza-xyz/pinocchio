#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccountState {
    /// Account is not yet initialized
    Uninitialized,

    /// Account is initialized; the account owner and/or delegate may perform
    /// permitted operations on this account
    Initialized,

    /// Account has been frozen by the mint freeze authority. Neither the
    /// account owner nor the delegate are able to perform operations on
    /// this account.
    Frozen,
}

impl From<u8> for AccountState {
    fn from(value: u8) -> Self {
        match value {
            0 => AccountState::Uninitialized,
            1 => AccountState::Initialized,
            2 => AccountState::Frozen,
            _ => panic!("invalid account state value: {value}"),
        }
    }
}

impl From<AccountState> for u8 {
    fn from(value: AccountState) -> Self {
        match value {
            AccountState::Uninitialized => 0,
            AccountState::Initialized => 1,
            AccountState::Frozen => 2,
        }
    }
}

/// Different kinds of accounts. Note that `Mint`, `TokenAccount`, and `Multisig`
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
    TokenAccount,
}

impl From<AccountType> for u8 {
    fn from(value: AccountType) -> Self {
        match value {
            AccountType::Uninitialized => 0,
            AccountType::Mint => 1,
            AccountType::TokenAccount => 2,
        }
    }
}
