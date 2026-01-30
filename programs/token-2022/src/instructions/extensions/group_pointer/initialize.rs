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

/// Initialize a new mint with a group pointer
///
/// Accounts expected by this instruction:
///`
///  0. `writable` The mint to initialize.
pub struct Initialize<'a, 'b> {
    /// Mint Account
    pub mint: &'a AccountView,
    /// Optional authority that can set the group address
    pub authority: Option<&'b Address>,
    /// Optional account address that holds the group
    pub group_address: Option<&'b Address>,
    /// Token Program
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

        // Encode discriminators (GroupPointer + Initialize)
        data[..2].copy_from_slice(&[
            ExtensionDiscriminator::GroupPointer as u8,
            Initialize::DISCRIMINATOR,
        ]);

        // write authority address bytes at offset [2..34]
        if let Some(authority) = self.authority {
            data[2..34].copy_from_slice(authority.to_bytes().as_ref());
        } else {
            data[2..34].copy_from_slice(&[0; 32]);
        }

        // write group_address address bytes at offset [34..66]
        if let Some(group_address) = self.group_address {
            data[34..66].copy_from_slice(group_address.to_bytes().as_ref());
        } else {
            data[34..66].copy_from_slice(&[0; 32]);
        }

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &instruction_accounts,
            data: data.as_ref(),
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
