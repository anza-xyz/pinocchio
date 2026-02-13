#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccountType {
    Uninitialized,
    Mint,
    Account,
}
