use solana_account_view::AccountView;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    AccountRole, InstructionView,
};
use solana_program_error::ProgramResult;

/// Withdraw funds from a nonce account.
///
/// The `u64` parameter is the lamports to withdraw, which must leave the
/// account balance above the rent exempt reserve or at zero.
///
/// ### Accounts:
///   0. `[WRITE]` Nonce account
///   1. `[WRITE]` Recipient account
///   2. `[]` Recent blockhashes sysvar
///   3. `[]` Rent sysvar
///   4. `[SIGNER]` Nonce authority
pub struct WithdrawNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a AccountView,

    /// Recipient account.
    pub recipient: &'a AccountView,

    /// Recent blockhashes sysvar.
    pub recent_blockhashes_sysvar: &'a AccountView,

    /// Rent sysvar.
    pub rent_sysvar: &'a AccountView,

    /// Nonce authority.
    pub authority: &'a AccountView,

    /// Lamports to withdraw.
    ///
    /// The account balance must be left above the rent exempt reserve
    /// or at zero.
    pub lamports: u64,
}

impl WithdrawNonceAccount<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountRole; 5] = [
            AccountRole::writable(self.account.address()),
            AccountRole::writable(self.recipient.address()),
            AccountRole::readonly(self.recent_blockhashes_sysvar.address()),
            AccountRole::readonly(self.rent_sysvar.address()),
            AccountRole::readonly_signer(self.authority.address()),
        ];

        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..12]: lamports
        let mut instruction_data = [0; 12];
        instruction_data[0] = 5;
        instruction_data[4..12].copy_from_slice(&self.lamports.to_le_bytes());

        let instruction = InstructionView {
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
