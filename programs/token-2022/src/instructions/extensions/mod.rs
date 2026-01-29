pub mod memo_transfer;
pub mod transfer_hook;

#[repr(u8)]
#[non_exhaustive]
pub enum ExtensionDiscriminator {
    MemoTransfer = 30,
    TransferHook = 36,
}
