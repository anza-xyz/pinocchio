use pinocchio::pubkey::Pubkey;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransferHookInstruction {
    Initialize,
    Update,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TransferHook {
    /// Authority that can set the transfer hook program id
    authority: Pubkey,
    /// Program that authorizes the transfer
    program_id: Pubkey,
}

/// Instruction data layout:
/// -  [0]:                 instruction discriminator (1 byte, u8)
/// -  [1]:                 instruction_type (1 byte, u8)
/// -  [2..34]:             authority (32 bytes, Pubkey)
/// -  [34..66]:            program_id (32 bytes, Pubkey)
pub mod offset_transfer_hook_initialize {
    pub const START: u8 = 2;
    pub const AUTHORITY_PUBKEY: u8 = 32;
    pub const PROGRAM_ID_PUBKEY: u8 = 32;
    pub const END: u8 = START + AUTHORITY_PUBKEY + PROGRAM_ID_PUBKEY;
}

/// Instruction data layout:
/// -  [0]:                 instruction discriminator (1 byte, u8)
/// -  [1]:                 instruction_type (1 byte, u8)
/// -  [2..34]:             program_id (32 bytes, Pubkey)          
pub mod offset_transfer_hook_update {
    pub const START: u8 = 2;
    pub const PROGRAM_ID_PUBKEY: u8 = 32;
    pub const END: u8 = START + PROGRAM_ID_PUBKEY;
}
