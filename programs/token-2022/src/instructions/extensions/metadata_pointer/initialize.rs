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

/// Initialize a new mint with a metadata pointer
///
/// Accounts expected by this instruction:
///
///  0. `[writable]` The mint to initialize.
pub struct Initialize<'a, 'b> {
    /// The mint to initialize with the metadata pointer extension.
    pub mint: &'a AccountView,
    /// Optional authority that can later update the metadata address.
    pub authority: Option<&'b Address>,
    /// Optional initial metadata address.
    pub metadata_address: Option<&'b Address>,
    /// Token program (Token-2022).
    pub token_program: &'b Address,
}

impl Initialize<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let instruction_accounts = [InstructionAccount::writable(self.mint.address())];

        let mut data = [0u8; 66];

        // Encode discriminators (Metadata + Initialize)
        data[..2].copy_from_slice(&[
            ExtensionDiscriminator::MetadataPointer as u8,
            Initialize::DISCRIMINATOR,
        ]);

        // write authority Address bytes at offset  [2..34]
        if let Some(authority) = self.authority {
            data[2..34].copy_from_slice(authority.to_bytes().as_ref());
        } else {
            data[2..34].copy_from_slice(&[0; 32]);
        }

        // write metadata_address Address bytes at offset [34..66 ]
        if let Some(metadata_address) = self.metadata_address {
            data[34..66].copy_from_slice(metadata_address.to_bytes().as_ref());
        } else {
            data[34..66].copy_from_slice(&[0; 32]);
        }

        // instruction
        let instruction = InstructionView {
            program_id: self.token_program,
            data: data.as_ref(),
            accounts: &instruction_accounts,
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
