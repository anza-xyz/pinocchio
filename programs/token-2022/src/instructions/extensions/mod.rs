pub mod default_account_state;
pub mod group_member_pointer;
pub mod group_pointer;
pub mod memo_transfer;
pub mod metadata_pointer;
pub mod permanent_delegate;
pub mod reallocate;
pub mod transfer_hook;

#[repr(u8)]
#[non_exhaustive]
pub enum ExtensionDiscriminator {
    DefaultAccountState = 28,
    Reallocate = 29,
    MemoTransfer = 30,
    PermanentDelegate = 35,
    TransferHook = 36,
    MetadataPointer = 39,
    GroupPointer = 40,
    GroupMemberPointer = 41,
}
