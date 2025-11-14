use {
    crate::{instructions::extensions::ExtensionDiscriminator, write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initialize a new mint with a group pointer
///
/// Accounts expected by this instruction:
///
///  0. `[writable]` The mint to initialize.
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
        let accounts = [InstructionAccount::writable(self.mint.address())];

        let mut data = [UNINIT_BYTE; 66];

        // Encode discriminators (GroupPointer + Initialize)
        write_bytes(
            &mut data[..2],
            &[
                ExtensionDiscriminator::GroupPointer as u8,
                Initialize::DISCRIMINATOR,
            ],
        );

        // write authority address bytes at offset [2..34]
        if let Some(authority) = self.authority {
            write_bytes(&mut data[2..34], authority.to_bytes().as_ref());
        } else {
            write_bytes(&mut data[2..34], &[0u8; 32]);
        }

        // write group_address address bytes at offset [34..66]
        if let Some(group_address) = self.group_address {
            write_bytes(&mut data[34..66], group_address.to_bytes().as_ref());
        } else {
            write_bytes(&mut data[34..66], &[0u8; 32]);
        }

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &accounts,
            data: unsafe { from_raw_parts(data.as_ptr() as _, data.len()) },
        };

        invoke(&instruction, &[self.mint])
    }
}
