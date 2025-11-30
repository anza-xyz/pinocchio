#[repr(u8)]
pub enum ExtensionDiscriminator {
    CpiGuard = 34,
    MemoTransfer = 30,
    PermanentDelegate = 35,
    ScaledUiAmount = 43,
    Pausable = 44,
    DefaultAccountState = 28,
    GroupPointer = 40,
    GroupMemberPointer = 41,
    TransferHook = 36,
    InterestBearingMint = 33,
    MetadataPointer = 39,
}
