pub mod memo_transfer;
pub mod metadata_pointer;

#[repr(u8)]
#[non_exhaustive]
pub enum ExtensionDiscriminator {
    MemoTransfer = 30,
    MetadataPointer = 39,
}
