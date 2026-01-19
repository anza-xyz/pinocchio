pub mod memo_transfer;
pub mod scaled_ui_amount;

#[repr(u8)]
#[non_exhaustive]
pub enum ExtensionDiscriminator {
    MemoTransfer = 30,
    ScaledUiAmount = 43,
}
