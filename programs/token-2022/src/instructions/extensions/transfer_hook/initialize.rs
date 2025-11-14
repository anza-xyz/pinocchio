use {
    crate::{instructions::extensions::ExtensionDiscriminator, write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initialize the Transfer Hook extension on a mint.
///
/// Expected accounts:
///
/// 0. `[writable]` The mint account to initialize the Transfer Hook extension.
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
        let accounts = [InstructionAccount::writable(self.mint_account.address())];

        let mut data = [UNINIT_BYTE; 66];

        // Encode discriminators (TransferHook + Initialize)
        write_bytes(
            &mut data[..2],
            &[
                ExtensionDiscriminator::TransferHook as u8,
                InitializeTransferHook::DISCRIMINATOR,
            ],
        );

        // Set authority at offset [2..34]
        if let Some(authority) = self.authority {
            write_bytes(&mut data[2..34], authority.to_bytes().as_ref());
        } else {
            write_bytes(&mut data[2..34], &[0; 32]);
        }

        // Set program_id at offset [34..66]
        if let Some(program_id) = self.program_id {
            write_bytes(&mut data[34..66], program_id.to_bytes().as_ref());
        } else {
            write_bytes(&mut data[34..66], &[0; 32]);
        }

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &accounts,
            data: unsafe { from_raw_parts(data.as_ptr() as _, data.len()) },
        };

        invoke(&instruction, &[self.mint_account])
    }
}
