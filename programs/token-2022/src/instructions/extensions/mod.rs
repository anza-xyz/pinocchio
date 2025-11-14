pub mod memo_transfer;
pub mod permanent_delegate;

#[repr(u8)]
#[non_exhaustive]
pub enum ExtensionDiscriminator {
    MemoTransfer = 30,
    PermanentDelegate = 35,
}
