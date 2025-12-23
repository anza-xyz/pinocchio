use pinocchio::pubkey::Pubkey;

/// Instruction data layout:
/// - [0]                        : Extension discriminator (1 byte)
/// - [1..33]                    : permanent delegate pubkey (32 bytes)
pub mod offset_permanent_delegate_initialize {
    pub const START: u8 = 1;
    pub const PERMANENT_DELEGATE_PUBKEY: u8 = 32;
    pub const END: u8 = START + PERMANENT_DELEGATE_PUBKEY;
}

/// Permanent delegate extension data for mints.
#[repr(C)]
pub struct PermanentDelegate {
    /// Optional permanent delegate for transferring or burning tokens
    delegate: Pubkey,
}
