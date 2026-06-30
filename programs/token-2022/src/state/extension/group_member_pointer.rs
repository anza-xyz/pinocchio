use {
    super::{sealed, ExtensionType, ExtensionValue},
    solana_address::Address,
    solana_nullable::MaybeNull,
};

/// Group member pointer extension data for mints (64 bytes).
///
/// Points to the account that holds the mint's token group member
/// configuration and names the authority permitted to update that pointer.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GroupMemberPointerExtension {
    pub authority: MaybeNull<Address>,
    pub member_address: MaybeNull<Address>,
}

impl GroupMemberPointerExtension {
    pub const LEN: usize = core::mem::size_of::<GroupMemberPointerExtension>();
}

impl sealed::Sealed for GroupMemberPointerExtension {}

// SAFETY: `GroupMemberPointerExtension` is repr(C), contains only
// `MaybeNull<Address>` fields which are repr(transparent) over `Address`
// (`[u8; 32]`), has no padding, and all bit patterns are valid.
unsafe impl ExtensionValue for GroupMemberPointerExtension {
    const TYPE: ExtensionType = ExtensionType::GroupMemberPointer;
}
