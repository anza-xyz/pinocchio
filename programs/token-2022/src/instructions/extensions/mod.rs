pub mod confidential_mint_burn;
pub mod default_account_state;
pub mod group_member_pointer;
pub mod group_pointer;
pub mod interest_bearing_mint;
pub mod memo_transfer;
pub mod metadata_pointer;
pub mod mint_close_authority;
pub mod permanent_delegate;
pub mod permissioned_burn;
pub mod scaled_ui_amount;
pub mod transfer_hook;

#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtensionDiscriminator {
    MintCloseAuthority = 25,
    DefaultAccountState = 28,
    MemoTransfer = 30,
    InterestBearingMint = 33,
    PermanentDelegate = 35,
    TransferHook = 36,
    MetadataPointer = 39,
    GroupPointer = 40,
    GroupMemberPointer = 41,
    ConfidentialMintBurn = 42,
    ScaledUiAmount = 43,
    PermissionedBurn = 46,
}
