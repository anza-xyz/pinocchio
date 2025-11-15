#[repr(u8)]
pub enum InstructionDiscriminatorMemoTransfer {
    Enable = 0,
    Disable = 1,
}

/// Instruction data layout:
/// - [0]                        : Extension discriminator (1 byte)
/// - [1]                        : Instruction discriminator (1 byte)
/// dev: Since only the instruction discriminator is used to toggle memo transfer states
/// and no additional parameters are required, `START == END`.
pub mod offset_memo_transfer {
    pub const START: u8 = 2;
    pub const END: u8 = START;
}

/// Models onchain `MemoTransfer` state.
/// Mirrors SPL Token-2022:
/// `pub struct MemoTransfer { pub require_incoming_transfer_memos: PodBool }`
#[repr(C)]
pub struct MemoTransfer {
    /// Indicates whether incoming transfers must include a memo.
    pub require_incoming_transfer_memos: bool,
}
