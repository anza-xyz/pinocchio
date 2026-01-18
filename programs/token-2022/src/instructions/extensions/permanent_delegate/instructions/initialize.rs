use {
    crate::instructions::extensions::{
        permanent_delegate::state::offset_permanent_delegate_initialize as OFFSET,
        ExtensionDiscriminator,
    },
    pinocchio::{
        account_info::AccountInfo,
        cpi::invoke,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        ProgramResult,
    },
};

/// Initialize the permanent delegate on a new mint.
///
/// Fails if the mint has already been initialized, so must be called before
/// `InitializeMint`.
///
/// The mint must have exactly enough space allocated for the base mint (82
/// bytes), plus 83 bytes of padding, 1 byte reserved for the account type,
/// then space required for this extension, plus any others.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint to initialize.
///
/// Data expected by this instruction:
///   Pubkey for the permanent delegate
pub struct InitializePermanentDelegate<'a, 'b> {
    /// The mint to initialize the permanent delegate
    pub mint: &'a AccountInfo,
    /// The public key for the account that can close the mint
    pub delegate: &'b Pubkey,
    /// Token Program
    pub token_program: &'b Pubkey,
}

impl InitializePermanentDelegate<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let account_metas = [AccountMeta::writable(self.mint.key())];

        let mut buffer = [0u8; OFFSET::END as usize];
        let data = initialize_instruction_data(&mut buffer, &self.delegate);

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data,
        };

        invoke(&instruction, &[self.mint])
    }
}

#[inline(always)]
fn initialize_instruction_data<'a>(buffer: &'a mut [u8], delegate: &Pubkey) -> &'a [u8] {
    let offset = OFFSET::START as usize;

    // Set discriminator
    buffer[..offset].copy_from_slice(&[ExtensionDiscriminator::PermanentDelegate as u8]);

    // Set delegate
    buffer[offset..offset + OFFSET::PERMANENT_DELEGATE_PUBKEY as usize].copy_from_slice(delegate);

    buffer
}
