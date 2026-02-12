use {
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::ProgramResult,
};

/// Creates the native mint.
///
/// This instruction only needs to be invoked once after deployment and is
/// permissionless, Wrapped SOL ([`crate::native_mint::id()`]) will not be
/// available until this instruction is successfully executed.
///
/// Accounts expected by this instruction:
///
///   0. `[writable,signer]` Funding account (must be a system account).
///   1. `[writable]` The native mint account to create.
///   2. `[]` System program for mint account funding.
pub struct CreateNativeMint<'a, 'b> {
    /// Funding account (must be a system account).
    pub payer: &'a AccountView,

    /// The native mint account to create.
    pub native_mint: &'a AccountView,

    /// System program for mint account funding.
    pub system_program: &'a AccountView,

    /// The token program.
    pub token_program: &'b Address,
}

impl CreateNativeMint<'_, '_> {
    pub const DISCRIMINATOR: u8 = 31;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        invoke_signed(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[
                    InstructionAccount::writable_signer(self.payer.address()),
                    InstructionAccount::writable(self.native_mint.address()),
                    InstructionAccount::readonly(self.system_program.address()),
                ],
                data: &[Self::DISCRIMINATOR],
            },
            &[self.payer, self.native_mint, self.system_program],
            signers,
        )
    }
}
