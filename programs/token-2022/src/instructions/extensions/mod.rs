pub mod confidential_transfer;
pub mod memo_transfer;

#[repr(u8)]
#[non_exhaustive]
pub enum ExtensionDiscriminator {
    MemoTransfer = 30,
    ConfidentialTransfer = 27,
}
