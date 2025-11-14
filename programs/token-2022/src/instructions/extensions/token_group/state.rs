use pinocchio::pubkey::Pubkey;

#[repr(u64)]
pub enum InstructionDiscriminatorTokenGroup {
    InitializeGroup = 288286683834380665, // [121, 113, 108, 39, 54, 51, 0, 4]
    UpdateGroupMaxSize = 7931435946663945580, // [108, 37, 171, 143, 248, 30, 18, 110]
    UpdateGroupAuthority = 14688734194668431777, // [161, 105, 88, 1, 237, 221, 216, 203]
    InitializeMember = 9688630243381616792, // [152, 32, 222, 176, 223, 237, 116, 134]
}

/// Instruction data layout:
/// - [0..8]                     : Instruction discriminator (8 bytes)
/// - [8..40]                    : update_authority pubkey (32 bytes)
/// - [40..48]                   : max_size (8 bytes)
pub mod offset_token_group_initialize_group {
    pub const START: u8 = 8;
    pub const UPDATE_AUTHORITY: u8 = 32;
    pub const MAX_SIZE: u8 = 8;
    pub const END: u8 = START + UPDATE_AUTHORITY + MAX_SIZE;
}

/// Instruction data layout:
/// - [0..8]                     : Instruction discriminator (8 bytes)
/// - [8..16]                    : max_size (8 bytes)
pub mod offset_token_group_update_max_size {
    pub const START: u8 = 8;
    pub const MAX_SIZE: u8 = 8;
    pub const END: u8 = START + MAX_SIZE;
}

/// Instruction data layout:
/// - [0..8]                     : Instruction discriminator (8 bytes)
/// - [8..40]                    : new_authority pubkey (32 bytes)
pub mod offset_token_group_update_authority {
    pub const START: u8 = 8;
    pub const NEW_AUTHORITY: u8 = 32;
    pub const END: u8 = START + NEW_AUTHORITY;
}

/// Instruction data layout:
/// - [0..8]                     : Instruction discriminator (8 bytes)
pub mod offset_token_group_initialize_member {
    pub const START: u8 = 8;
    pub const END: u8 = START;
}

/// Data struct for a `TokenGroup`
#[repr(C)]
pub struct TokenGroup {
    /// The authority that can sign to update the group
    update_authority: Pubkey,
    /// The associated mint, used to counter spoofing to be sure that group
    /// belongs to a particular mint
    mint: Pubkey,
    /// The current number of group members
    size: u64,
    /// The maximum number of group members
    max_size: u64,
}
