pub mod memo_transfer;
pub mod pausable;

#[repr(u8)]
#[non_exhaustive]
pub enum ExtensionDiscriminator {
    MemoTransfer = 30,
    Pausable = 44,
}
