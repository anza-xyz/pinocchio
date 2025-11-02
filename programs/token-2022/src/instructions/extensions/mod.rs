#[repr(u8)]
pub enum ExtensionDiscriminator {
    /// Default Account State extension
    DefaultAccountState = 28,
    /// Memo Transfer extension
    MemoTransfer = 30,
    /// Interest-Bearing Mint extension
    InterestBearingMint = 33,
    /// CPI Guard extension
    CpiGuard = 34,
    /// Permanent Delegate extension
    PermanentDelegate = 35,
    /// Transfer Hook extension
    TransferHook = 36,
    /// Metadata Pointer extension
    MetadataPointer = 39,
    /// Group Pointer extension
    GroupPointer = 40,
    /// Group Member Pointer extension
    GroupMemberPointer = 41,
    /// Scaled UI Amount extension
    ScaledUiAmount = 43,
    /// Pausable extension
    Pausable = 44,
}
