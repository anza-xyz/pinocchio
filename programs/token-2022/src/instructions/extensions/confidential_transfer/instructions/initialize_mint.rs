use core::slice::from_raw_parts;

use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView};
use solana_program_error::ProgramResult;

use crate::{UNINIT_BYTE, instructions::ExtensionDiscriminator, write_bytes};

//extensions::ExtensionDiscriminator, write_bytes, UNINIT_BYTE};

/// Initialize confidential transfers for a mint
///
/// The instruction requires no signers and MUST be included within the same Transaction
/// as `instructions::initialize_mint`. Otherwise another party can initialize the configuration.
///
/// The instruction fails if the `instructions::initialize_mint`
/// instruction has already executed for the mint.
///
/// Accounts expected by this instruction:
///
///     0. `[writable]` The SPL Token mint.
///
pub struct InitializeMint<'a, 'data> {
    /// Token program to invoke
    pub token_program: &'a Address,
    /// Token Mint account
    pub mint: &'a AccountView,
    /// Data required by the instruction
    /// Authority to modify the confidential mint configuration, and to approve new account
    pub authority: Option<&'data Address>,
    /// New authority to decode any transfer amount in a confidential transfer.
    pub auditor_elgamal_pubkey: Option<&'data Address>,
    /// Determines whether the newly configured accounts must be approved by the
    /// `authority` before they may be used by the user.
    pub auto_approve_new_account: bool,
}

impl InitializeMint<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let accounts = [InstructionAccount::writable(self.mint.address())];

        //extenson type discriminator(1) +
        //extension instruction discriminator(1) +
        //authority(32) +
        //auditor(32) +
        //auto approve new account(1)
        let mut data = [UNINIT_BYTE; 67];

        //set discriminators (ConfidentialTransfer + Initialize)
        write_bytes(
            &mut data[..2],
            &[
                ExtensionDiscriminator::ConfidentialTransfer as u8,
                InitializeMint::DISCRIMINATOR,
            ],
        );

        // write authority address bytes at offset [2..34]
        if let Some(authority) = self.authority {
            write_bytes(&mut data[2..34], authority.to_bytes().as_ref());
        } else {
            write_bytes(&mut data[2..34], &[0u8; 32]);
        };

        // write auto_approve_new_account
        if self.auto_approve_new_account {
            write_bytes(&mut data[34..35], &[1]);
        } else {
            write_bytes(&mut data[34..35], &[0]);
        };

        // write auditor_elgamal_pubkey
        if let Some(auditor) = self.auditor_elgamal_pubkey {
            write_bytes(&mut data[35..67], auditor.to_bytes().as_ref());
        } else {
            write_bytes(&mut data[35..67], &[0u8; 32]);
        };

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &accounts,
            data: unsafe { from_raw_parts(data.as_ptr() as *const u8, data.len()) },
        };

        invoke(&instruction, &[self.mint])
    }
}
