pub mod default_account_state;
pub mod memo_transfer;

#[repr(u8)]
#[non_exhaustive]
pub enum ExtensionDiscriminator {
    DefaultAccountState = 28,
    MemoTransfer = 30,
}
