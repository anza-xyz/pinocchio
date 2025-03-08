use {
    pinocchio::{
        account_info::AccountInfo,
        instruction::{AccountMeta, Instruction, Signer},
        pubkey::Pubkey,
        ProgramResult,
    },
    pinocchio_cpi::invoke_signed,
};

/// Assign account to a program
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Assigned account public key
pub struct Assign<'a, 'b> {
    /// Account to be assigned.
    pub account: &'a AccountInfo,

    /// Program account to assign as owner.
    pub owner: &'b Pubkey,
}

impl Assign<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable_signer(self.account.key())];

        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..36]: owner pubkey
        let mut instruction_data = [0; 36];
        instruction_data[0] = 1;
        instruction_data[4..36].copy_from_slice(self.owner.as_ref());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(&instruction, &[self.account], signers)
    }
}
