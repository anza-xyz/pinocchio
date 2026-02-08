pub mod default_account_state;
pub mod memo_transfer;
pub mod transfer_hook;

#[repr(u8)]
#[non_exhaustive]
pub enum ExtensionDiscriminator {
    DefaultAccountState = 28,
    MemoTransfer = 30,
    TransferHook = 36,
}
