use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::{cpi::invoke, AccountRole, InstructionView};
use solana_program_error::ProgramResult;

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
    pub account: &'a AccountView,

    /// Recent blockhashes sysvar.
    pub recent_blockhashes_sysvar: &'a AccountView,

    /// Rent sysvar.
    pub rent_sysvar: &'a AccountView,

    /// Indicates the entity authorized to execute nonce
    /// instruction on the account
    pub authority: &'b Address,
}

impl InitializeNonceAccount<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // account metadata
        let account_metas: [AccountRole; 3] = [
            AccountRole::writable(self.account.address()),
            AccountRole::readonly(self.recent_blockhashes_sysvar.address()),
            AccountRole::readonly(self.rent_sysvar.address()),
        ];

        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..36]: authority address
        let mut instruction_data = [0; 36];
        instruction_data[0] = 6;
        instruction_data[4..36].copy_from_slice(self.authority.as_array());

        let instruction = InstructionView {
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
