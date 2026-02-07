pub mod memo_transfer;
pub mod transfer_fee;

#[repr(u8)]
#[non_exhaustive]
pub enum ExtensionDiscriminator {
    TransferFee = 26,

    MemoTransfer = 30,
}
