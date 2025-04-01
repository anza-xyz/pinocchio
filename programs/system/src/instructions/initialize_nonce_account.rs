use pinocchio::{
    account_info::AccountInfo,
    Address,
    cpi::invoke,
    instruction::{AccountMeta, Instruction},
    ProgramResult,
};

/// Drive state of Uninitialized nonce account to Initialized, setting the nonce value.
///
/// The [`Address`] parameter specifies the entity authorized to execute nonce
/// instruction on the account
///
/// No signatures are required to execute this instruction, enabling derived
/// nonce account addresses.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[]` Recent blockhashes sysvar
///   2. `[]` Rent sysvar
pub struct InitializeNonceAccount<'a, 'b> {
    /// Nonce account.
    pub account: &'a AccountInfo,

    /// Recent blockhashes sysvar.
    pub recent_blockhashes_sysvar: &'a AccountInfo,

    /// Rent sysvar.
    pub rent_sysvar: &'a AccountInfo,

    /// Indicates the entity authorized to execute nonce
    /// instruction on the account
    pub authority: &'b Address,
}

impl InitializeNonceAccount<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly(self.recent_blockhashes_sysvar.key()),
            AccountMeta::readonly(self.rent_sysvar.key()),
        ];

        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..36]: authority address
        let mut instruction_data = [0; 36];
        instruction_data[0] = 6;
        instruction_data[4..36].copy_from_slice(self.authority.as_array());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke(
            &instruction,
            &[
                self.account,
                self.recent_blockhashes_sysvar,
                self.rent_sysvar,
            ],
        )
    }
}
