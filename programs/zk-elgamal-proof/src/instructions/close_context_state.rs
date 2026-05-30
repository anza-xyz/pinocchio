use {
    crate::ContextStateInfo,
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::ProgramResult,
};

/// Close a zero-knowledge proof context state.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The proof context account to close
///   1. `[writable]` The destination account for lamports
///   2. `[signer]` The context account's owner
pub struct CloseContextState<'a, 'b> {
    /// Context state to close
    pub context_state_info: ContextStateInfo<'a>,
    /// Destination account for lamports
    pub destination_account: &'b AccountView,
}

impl CloseContextState<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let instruction_accounts: [InstructionAccount; 3] = [
            InstructionAccount::writable(self.context_state_info.context_state_account.address()),
            InstructionAccount::writable(self.destination_account.address()),
            InstructionAccount::readonly_signer(
                self.context_state_info.context_state_authority.address(),
            ),
        ];

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            data: &[0],
        };

        invoke_signed(
            &instruction,
            &[
                self.context_state_info.context_state_account,
                self.destination_account,
                self.context_state_info.context_state_authority,
            ],
            signers,
        )
    }
}
