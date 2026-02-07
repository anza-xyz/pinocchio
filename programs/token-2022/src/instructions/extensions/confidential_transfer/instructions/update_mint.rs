use {
    crate::{instructions::ExtensionDiscriminator, write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::ProgramResult,
};

/// Updates the confidential transfer mint configuration for a rent
///
/// Use `instructions:set_authority` to update the confidential transfer mint
/// authority
///
/// Accounts expected by this instruction:
///
/// 0. `[writable]` The SPL Token mint.
/// 1. `[Signer]` Confidential transfer mint authority
///
/// Data expected by this instruction:
///     0. `auto_approve_new_account` Whether a new account should be approved
///        by the authority
///     1. `auditor_elgamal_pubkey` An optional auditor
pub struct UpdateMint<'a, 'data> {
    /// Token program to invoke
    pub token_program: &'a Address,
    /// Mint account
    pub mint: &'a AccountView,
    // Confidential transfer mint authority
    pub authority: &'a AccountView,
    /// New authority to decode any transfer amount in a confidential transfer.
    pub auditor_elgamal_pubkey: Option<&'data Address>,
    /// Determines if newly configured accounts must be approved by
    /// the `authority` before they may be used by the user.
    pub auto_approve_new_accounts: bool,
}

impl UpdateMint<'_, '_> {
    pub const DISCRIMINATOR: u8 = 1;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ];

        // extension discriminator(1) +
        // extension instruction discriminator(1) +
        // auditor_elgamal_pubkey(32) +
        // auto_approve_new_account(1)
        let mut data = [UNINIT_BYTE; 35];

        // set discriminators (ConfidentialTransfer + UpdateMint)
        write_bytes(
            &mut data[..2],
            &[
                ExtensionDiscriminator::ConfidentialTransfer as u8,
                UpdateMint::DISCRIMINATOR,
            ],
        );

        // set auto_approve_new_account
        if self.auto_approve_new_accounts {
            write_bytes(&mut data[2..3], &[1]);
        } else {
            write_bytes(&mut data[2..3], &[0]);
        };

        // set auditor_elgamal_pubkey
        if let Some(auditor) = self.auditor_elgamal_pubkey {
            write_bytes(&mut data[3..35], auditor.to_bytes().as_ref());
        } else {
            write_bytes(&mut data[3..35], &[0u8]);
        };

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &accounts,
            data: unsafe { from_raw_parts(data.as_ptr() as _, data.len()) },
        };

        invoke_signed_with_bounds::<2>(&instruction, &[self.mint, self.authority], signers)
    }
}
