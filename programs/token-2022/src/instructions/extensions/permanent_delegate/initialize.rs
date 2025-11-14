use {
    crate::instructions::extensions::ExtensionDiscriminator,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
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
    pub mint: &'a AccountView,
    /// The public key for the account that can close the mint
    pub delegate: &'b Address,
    /// Token Program
    pub token_program: &'b Address,
}

impl InitializePermanentDelegate<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let instruction_accounts = [InstructionAccount::writable(self.mint.address())];

        let mut data = [0u8; 34];

        // write discriminator at index 1
        data[..1].copy_from_slice(&[ExtensionDiscriminator::PermanentDelegate as u8]);

        // write delegate address bytes at offset [2..34]
        data[2..34].copy_from_slice(self.delegate.to_bytes().as_ref());

        // ix
        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &instruction_accounts,
            data: data.as_ref(),
        };

        invoke(&instruction, &[self.mint])
    }
}
