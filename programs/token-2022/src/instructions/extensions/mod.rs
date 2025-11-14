pub mod default_account_state;
pub mod memo_transfer;
pub mod permanent_delegate;
pub mod transfer_hook;
pub mod metadata_pointer;

#[repr(u8)]
#[non_exhaustive]
pub enum ExtensionDiscriminator {
    DefaultAccountState = 28,
    MemoTransfer = 30,
    PermanentDelegate = 35,
    TransferHook = 36,
    MetadataPointer = 39,
}
