use pinocchio::pubkey::Pubkey;

#[repr(u8)]
pub enum InstructionDiscriminatorMetadataPointer {
    Initialize = 0,
    Update = 1,
}

/// Instruction data layout:
/// - [0]                        : Extension discriminator (1 byte)
/// - [1]                        : Instruction discriminator (1 byte)
/// - [2..34]                    : authority pubkey (32 bytes)
/// - [34..66]                   : metadata_address pubkey (32 bytes)
pub mod offset_metadata_pointer_initialize {
    pub const START: u8 = 2;
    pub const AUTHORITY_PUBKEY: u8 = 32;
    pub const METADATA_ADDRESS_PUBKEY: u8 = 32;
    pub const END: u8 = START + AUTHORITY_PUBKEY + METADATA_ADDRESS_PUBKEY;
}

/// Instruction data layout:
/// - [0]                        : Extension discriminator (1 byte)
/// - [1]                        : Instruction discriminator (1 byte)
/// - [2..34]                    : metadata_address pubkey (32 bytes)
pub mod offset_metadata_pointer_update {
    pub const START: u8 = 2;
    pub const METADATA_ADDRESS_PUBKEY: u8 = 32;
    pub const END: u8 = START + METADATA_ADDRESS_PUBKEY;
}

/// Metadata pointer extension data for mints.
#[repr(C)]
pub struct MetadataPointer {
    /// Authority that can set the metadata address
    pub authority: Pubkey,
    /// Account address that holds the metadata
    pub metadata_address: Pubkey,
}
