use solana_account_view::AccountView;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    AccountRole, InstructionView,
};
use solana_program_error::ProgramResult;

/// Allocate space in a (possibly new) account without funding.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` New account
pub struct Allocate<'a> {
    /// Account to be assigned.
    pub account: &'a AccountView,

    /// Number of bytes of memory to allocate.
    pub space: u64,
}

impl Allocate<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountRole; 1] =
            [AccountRole::writable_signer(self.account.address())];

        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..12]: space
        let mut instruction_data = [0; 12];
        instruction_data[0] = 8;
        instruction_data[4..12].copy_from_slice(&self.space.to_le_bytes());

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(&instruction, &[self.account], signers)
    }
}
