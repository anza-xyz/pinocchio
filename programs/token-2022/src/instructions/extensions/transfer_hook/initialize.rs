use {
    crate::instructions::extensions::ExtensionDiscriminator,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::ProgramResult,
};

pub struct InitializeTransferHook<'a, 'b> {
    /// Mint Account to initialize.
    pub mint_account: &'a AccountView,
    /// Optional authority that can set the transfer hook program id
    pub authority: Option<&'b Address>,
    /// Program that authorizes the transfer
    pub program_id: Option<&'b Address>,
    /// Token Program
    pub token_program: &'b Address,
}

impl InitializeTransferHook<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let instruction_accounts = [InstructionAccount::writable(self.mint_account.address())];

        let mut data = [0u8; 66];

        // Encode discriminators (TransferHook + Initialize)
        data[..2].copy_from_slice(&[
            ExtensionDiscriminator::TransferHook as u8,
            InitializeTransferHook::DISCRIMINATOR,
        ]);

        // Set authority at offset [2..34]
        if let Some(x) = self.authority {
            data[2..34].copy_from_slice(x.to_bytes().as_ref());
        } else {
            data[2..34].copy_from_slice(&[0; 32]);
        }

        // Set program_id at offset [34..66]
        if let Some(x) = self.program_id {
            data[34..66].copy_from_slice(x.to_bytes().as_ref());
        } else {
            data[34..66].copy_from_slice(&[0; 32]);
        }

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &instruction_accounts,
            data: data.as_ref(),
        };

        invoke_signed(&instruction, &[self.mint_account], signers)
    }
}
