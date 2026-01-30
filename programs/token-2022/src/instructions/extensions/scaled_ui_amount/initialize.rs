use {
    crate::instructions::extensions::ExtensionDiscriminator,
    core::slice,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke_with_bounds, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

pub struct Initialize<'a, 'b> {
    /// Mint account to initialize
    pub mint_account: &'a AccountView,
    /// The public key for the account that can update the multiplier
    pub authority: Option<&'b Address>,
    /// The initial multiplier
    pub multiplier: f64,
    /// Token Program
    pub token_program: &'b Address,
}

impl Initialize<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let instruction_accounts = [InstructionAccount::writable(self.mint_account.address())];

        let authority = match self.authority {
            Some(auth) => auth,
            None => &Address::default(),
        };

        let data = &mut [0; 42];
        data[0] = ExtensionDiscriminator::ScaledUiAmount as u8;
        data[1] = Self::DISCRIMINATOR;
        data[2..34].copy_from_slice(authority.as_ref());
        data[34..42].copy_from_slice(&self.multiplier.to_le_bytes());

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &instruction_accounts,
            data,
        };

        let account_views = [self.mint_account];

        invoke_with_bounds::<1>(&instruction, unsafe {
            slice::from_raw_parts(account_views.as_ptr() as _, 1)
        })
    }
}
