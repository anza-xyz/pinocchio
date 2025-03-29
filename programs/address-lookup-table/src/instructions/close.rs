use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

/// Close an address lookup table account
///
/// # Account references
///   0. `[WRITE]` Address lookup table account to close
///   1. `[SIGNER]` Current authority
///   2. `[WRITE]` Recipient of closed account lamports
pub struct Close<'a> {
    /// Address lookup table account to close
    pub lookup_table: &'a AccountInfo,
    /// Current authority
    pub authority: &'a AccountInfo,
    ///  Recipient of closed account lamports
    pub recipient: &'a AccountInfo,
}

impl Close<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.lookup_table.key()),
            AccountMeta::readonly_signer(self.authority.key()),
            AccountMeta::writable(self.recipient.key()),
        ];

        // Instruction data:
        // - [0]: Instruction discriminator (1 byte, u8) (4 for Close)

        let instruction_data = [4u8];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(
            &instruction,
            &[
                self.lookup_table,
                self.authority,
                self.recipient,
            ],
            signers,
        )
    }
}
