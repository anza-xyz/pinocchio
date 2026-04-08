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
