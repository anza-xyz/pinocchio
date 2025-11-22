use {
    crate::{instructions::extensions::ExtensionDiscriminator, write_bytes, UNINIT_BYTE},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initialize the Scaled UI Amount extension on a mint account.
///
/// Expected accounts:
///
/// 0. `[writable]` The mint account to initialize with the Scaled UI Amount
///    extension.
pub struct Initialize<'a, 'b> {
    /// The mint account to initialize with the Scaled UI Amount extension.
    pub mint_account: &'a AccountView,
    /// The authority that can update the multiplier.
    pub authority: Option<&'b Address>,
    /// The initial multiplier value.
    pub multiplier: f64,
    /// Token program (Token-2022).
    pub token_program: &'b Address,
}

impl Initialize<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let accounts = [InstructionAccount::writable(self.mint_account.address())];

        let authority = match self.authority {
            Some(auth) => auth,
            None => &Address::default(),
        };

        let mut data = [UNINIT_BYTE; 42];
        write_bytes(
            &mut data[0..1],
            &[ExtensionDiscriminator::ScaledUiAmount as u8],
        );
        write_bytes(&mut data[1..2], &[Self::DISCRIMINATOR]);
        write_bytes(&mut data[2..34], authority.as_ref());
        write_bytes(&mut data[34..42], &self.multiplier.to_le_bytes());
        let data = unsafe { &*(data.as_ptr() as *const [u8; 42]) };

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &accounts,
            data,
        };

        invoke(&instruction, &[self.mint_account])
    }
}
