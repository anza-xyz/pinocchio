use {
    crate::{instructions::extensions::ExtensionDiscriminator, write_bytes, UNINIT_BYTE},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initialize the Pausable extension on a mint.
///
/// This instruction must be called after creating the mint but before initializing it.
///
/// Expected accounts:
///
/// 0. `[writable]` The mint account to initialize.
pub struct InitializePausable<'a, 'b> {
    /// The mint account to initialize.
    ///
    /// Note: This applies to the **Mint**, allowing the specified authority
    /// to pause all transfer/burn operations globally.
    pub mint: &'a AccountView,

    /// The address that will have the authority to pause/resume the mint.
    pub authority: &'a Address,

    /// Token program.
    pub token_program: &'b Address,
}

impl InitializePausable<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let &Self {
            mint,
            authority,
            token_program,
            ..
        } = self;

        let accounts = [InstructionAccount::writable(mint.address())];

        // build instruction data for Pausable
        // Layout: [Extension + Sub-Instruction + Authority]
        let mut data = [UNINIT_BYTE; 1 + 1 + 32];
        write_bytes(&mut data[0..1], &[ExtensionDiscriminator::Pausable as u8]);
        write_bytes(&mut data[1..2], &[Self::DISCRIMINATOR]);
        write_bytes(&mut data[2..34], authority.as_ref());

        let data = unsafe { &*(data.as_ptr() as *const [u8; 1 + 1 + 32]) };

        // build instruction for Pausable
        let instruction = InstructionView {
            program_id: token_program,
            data,
            accounts: &accounts,
        };

        cpi::invoke(&instruction, &[self.mint])
    }
}
