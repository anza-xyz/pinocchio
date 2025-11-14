use {
    crate::{instructions::extensions::ExtensionDiscriminator, write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
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
        let accounts = [InstructionAccount::writable(self.mint.address())];

        let mut data = [UNINIT_BYTE; 66];

        // Encode discriminators (Metadata + Initialize)
        write_bytes(
            &mut data[..2],
            &[
                ExtensionDiscriminator::MetadataPointer as u8,
                Initialize::DISCRIMINATOR,
            ],
        );

        // write authority Address bytes at offset  [2..34]
        if let Some(authority) = self.authority {
            write_bytes(&mut data[2..34], authority.to_bytes().as_ref());
        } else {
            write_bytes(&mut data[2..34], &[0; 32]);
        }

        // write metadata_address Address bytes at offset [34..66 ]
        if let Some(metadata_address) = self.metadata_address {
            write_bytes(&mut data[34..66], metadata_address.to_bytes().as_ref());
        } else {
            write_bytes(&mut data[34..66], &[0; 32]);
        }

        // instruction
        let instruction = InstructionView {
            program_id: self.token_program,
            data: unsafe { from_raw_parts(data.as_ptr() as _, data.len()) },
            accounts: &accounts,
        };

        invoke(&instruction, &[self.mint])
    }
}
