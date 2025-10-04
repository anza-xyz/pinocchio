use pinocchio::{
    account::AccountView,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    Address, ProgramResult,
};

/// Assign account to a program
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Assigned account address
pub struct Assign<'a, 'b> {
    /// Account to be assigned.
    pub account: &'a AccountView,

    /// Program account to assign as owner.
    pub owner: &'b Address,
}

impl Assign<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 1] =
            [AccountMeta::writable_signer(self.account.address())];

        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..36]: owner address
        let mut instruction_data = [0; 36];
        instruction_data[0] = 1;
        instruction_data[4..36].copy_from_slice(self.owner.as_ref());

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(&instruction, &[self.account], signers)
    }
}
