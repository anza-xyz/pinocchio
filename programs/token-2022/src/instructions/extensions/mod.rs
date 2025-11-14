pub mod cpi_guard;
pub mod default_account_state;
pub mod group_member_pointer;
pub mod group_pointer;
pub mod interest_bearing_mint;
pub mod memo_transfer;
pub mod metadata_pointer;
pub mod pausable;
pub mod permanent_delegate;
pub mod scaled_ui_amount;
pub mod token_group;
pub mod transfer_hook;

#[repr(u8)]
pub(crate) enum ExtensionDiscriminator {
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
