pub mod memo_transfer;

#[repr(u8)]
pub enum ExtensionDiscriminator {
    MemoTransfer = 30,
}
