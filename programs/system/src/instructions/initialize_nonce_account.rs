use {
    pinocchio::{
        account_info::AccountInfo,
        instruction::{AccountMeta, Instruction, Signer},
        pubkey::Pubkey,
        ProgramResult,
    },
    pinocchio_cpi::invoke_signed,
};

/// Drive state of Uninitialized nonce account to Initialized, setting the nonce value.
///
/// The `Pubkey` parameter specifies the entity authorized to execute nonce
/// instruction on the account
///
/// No signatures are required to execute this instruction, enabling derived
/// nonce account addresses.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[]` RecentBlockhashes sysvar
///   2. `[]` Rent sysvar
pub struct InitializeNonceAccount<'a, 'b> {
    /// Nonce account.
    pub account: &'a AccountInfo,

    /// RecentBlockhashes sysvar.
    pub recent_blockhashes_sysvar: &'a AccountInfo,

    /// Rent sysvar.
    pub rent_sysvar: &'a AccountInfo,

    /// Lamports to withdraw.
    ///
    /// The account balance muat be left above the rent exempt reserve
    /// or at zero.
    pub authority: &'b Pubkey,
}

impl InitializeNonceAccount<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly(self.recent_blockhashes_sysvar.key()),
            AccountMeta::readonly(self.rent_sysvar.key()),
        ];

        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..36]: authority pubkey
        let mut instruction_data = [0; 36];
        instruction_data[0] = 6;
        instruction_data[4..36].copy_from_slice(self.authority);

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(
            &instruction,
            &[
                self.account,
                self.recent_blockhashes_sysvar,
                self.rent_sysvar,
            ],
            signers,
        )
    }
}
