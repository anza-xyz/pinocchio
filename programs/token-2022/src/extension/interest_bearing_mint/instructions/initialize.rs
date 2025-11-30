use {
    crate::extension::interest_bearing_mint::state::interest_bearing_mint_initialize_instruction_data,
    core::slice,
    pinocchio::{
        account_info::AccountInfo,
        cpi::invoke_signed,
        instruction::{AccountMeta, Instruction, Signer},
        pubkey::Pubkey,
        ProgramResult,
    },
};

/// Initialize a new mint with interest accrual
///
/// Accounts expected by this instruction:
///
///  0. `[writable]` The mint to initialize.
pub struct Initialize<'a> {
    /// Mint Account
    pub mint: &'a AccountInfo,
    /// Optional authority that can set the interest rate
    pub rate_authority: Option<&'a Pubkey>,
    /// The initial interest rate
    pub rate: i16,
    /// Token Program
    pub token_program: &'a Pubkey,
}

impl Initialize<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            mint,
            rate_authority,
            rate,
            token_program,
        } = self;

        let account_metas = [AccountMeta::writable(mint.key())];

        let data = interest_bearing_mint_initialize_instruction_data(rate_authority, rate);

        let instruction = Instruction {
            program_id: token_program,
            accounts: &account_metas,
            data: unsafe { slice::from_raw_parts(data.as_ptr() as _, data.len()) },
        };

        invoke_signed(&instruction, &[mint], signers)
    }
}
