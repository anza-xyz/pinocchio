use pinocchio::pubkey::Pubkey;

#[repr(u8)]
pub enum InstructionDiscriminatorGroupMemberPointer {
    Initialize = 0,
    Update = 1,
}

/// Instruction data layout:
/// - [0]                        : Extension discriminator (1 byte)
/// - [1]                        : Initialize discriminator (1 byte)
/// - [2..34]                    : authority pubkey (32 bytes)
/// - [34..66]                   : member_address pubkey (32 bytes)
pub mod offset_group_member_pointer_initialize {
    pub const START: u8 = 2;
    pub const AUTHORITY_PUBKEY: u8 = 32;
    pub const MEMBER_ADDRESS_PUBKEY: u8 = 32;
    pub const END: u8 = START + AUTHORITY_PUBKEY + MEMBER_ADDRESS_PUBKEY;
}

/// Instruction data layout:
/// -  [0]: Extension discriminator (1 byte)
/// -  [1]: Instruction discriminator (1 byte)
/// -  [2..34]: member_address pubkey (optional, 32 bytes)
pub mod offset_group_member_pointer_update {
    pub const START: u8 = 2;
    pub const MEMBER_ADDRESS_PUBKEY: u8 = 32;
    pub const END: u8 = START + MEMBER_ADDRESS_PUBKEY;
}

#[repr(C)]
pub struct GroupMemberPointer {
    /// Authority that can set the member address
    authority: Pubkey,
    /// Account address that holds the member
    member_address: Pubkey,
}
