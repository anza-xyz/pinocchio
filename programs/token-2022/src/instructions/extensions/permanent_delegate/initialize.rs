use {
    crate::{instructions::extensions::ExtensionDiscriminator, write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
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
        let accounts = [InstructionAccount::writable(self.mint.address())];

        let mut data = [UNINIT_BYTE; 33];

        // write discriminator at index 1
        write_bytes(
            &mut data[..1],
            &[ExtensionDiscriminator::PermanentDelegate as u8],
        );

        // write delegate address bytes at offset [1..33]
        write_bytes(&mut data[1..33], self.delegate.to_bytes().as_ref());

        // ix
        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &accounts,
            data: unsafe { from_raw_parts(data.as_ptr() as _, data.len()) },
        };

        invoke(&instruction, &[self.mint])
    }
}
