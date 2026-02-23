use solana_program_error::ProgramError;

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

impl TryFrom<u8> for AccountState {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AccountState::Uninitialized),
            1 => Ok(AccountState::Initialized),
            2 => Ok(AccountState::Frozen),
            _ => Err(ProgramError::InvalidAccountData),
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
