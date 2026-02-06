pub mod immutable_owner;
pub mod memo_transfer;

#[repr(u8)]
#[non_exhaustive]
pub enum ExtensionDiscriminator {
    ImmutableOwner = 22,
    MemoTransfer = 30,
}
