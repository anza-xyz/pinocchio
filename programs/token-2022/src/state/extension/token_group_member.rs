use {
    super::{sealed, ExtensionType, ExtensionValue},
    solana_address::Address,
    solana_zero_copy::unaligned::U64,
};

/// Token group member extension data for mints (72 bytes).
///
/// Identifies the mint as a member of a group and records its member number.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TokenGroupMemberExtension {
    pub mint: Address,
    pub group: Address,
    pub member_number: U64,
}

impl TokenGroupMemberExtension {
    pub const LEN: usize = core::mem::size_of::<TokenGroupMemberExtension>();
}

impl sealed::Sealed for TokenGroupMemberExtension {}

// SAFETY: `TokenGroupMemberExtension` is repr(C), contains only `Address`
// fields (`[u8; 32]`) and a `U64` (repr(transparent) over `[u8; 8]`), has no
// padding, and all bit patterns are valid.
unsafe impl ExtensionValue for TokenGroupMemberExtension {
    const TYPE: ExtensionType = ExtensionType::TokenGroupMember;
}
