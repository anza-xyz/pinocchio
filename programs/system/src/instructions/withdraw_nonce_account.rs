use {
    pinocchio::{
        account_info::AccountInfo,
        instruction::{AccountMeta, Instruction, Signer},
        ProgramResult,
    },
    pinocchio_cpi::invoke_signed,
};

/// Withdraw funds from a nonce account.
///
/// The `u64` parameter is the lamports to withdraw, which must leave the
/// account balance above the rent exempt reserve or at zero.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[WRITE]` Recipient account
///   2. `[]` RecentBlockhashes sysvar
///   3. `[]` Rent sysvar
///   4. `[SIGNER]` Nonce authority
pub struct WithdrawNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a AccountInfo,

    /// Recipient account.
    pub recipient: &'a AccountInfo,

    /// RecentBlockhashes sysvar.
    pub recent_blockhashes_sysvar: &'a AccountInfo,

    /// Rent sysvar.
    pub rent_sysvar: &'a AccountInfo,

    /// Nonce authority.
    pub authority: &'a AccountInfo,

    /// Lamports to withdraw.
    ///
    /// The account balance muat be left above the rent exempt reserve
    /// or at zero.
    pub lamports: u64,
}

impl WithdrawNonceAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 5] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::writable(self.recipient.key()),
            AccountMeta::readonly(self.recent_blockhashes_sysvar.key()),
            AccountMeta::readonly(self.rent_sysvar.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..12]: lamports
        let mut instruction_data = [0; 12];
        instruction_data[0] = 5;
        instruction_data[4..12].copy_from_slice(&self.lamports.to_le_bytes());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(
            &instruction,
            &[
                self.account,
                self.recipient,
                self.recent_blockhashes_sysvar,
                self.rent_sysvar,
                self.authority,
            ],
            signers,
        )
    }
}
