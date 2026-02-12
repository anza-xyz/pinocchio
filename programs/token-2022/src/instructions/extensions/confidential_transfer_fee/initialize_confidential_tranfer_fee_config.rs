use {
    crate::{instructions::ExtensionDiscriminator, write_bytes, ELGAMAL_PUBKEY_LEN, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initializes confidential transfer fees for a mint.
///
/// The `ConfidentialTransferFeeInstruction::InitializeConfidentialTransferFeeConfig`
/// instruction requires no signers and MUST be included within the same
/// Transaction as `TokenInstruction::InitializeMint`. Otherwise another
/// party can initialize the configuration.
///
/// The instruction fails if the `TokenInstruction::InitializeMint`
/// instruction has already executed for the mint.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The SPL Token mint.
pub struct InitializeConfidentialTransferFeeConfig<'a, 'data> {
    /// The Token mint
    pub mint: &'a AccountView,
    /// The token program
    pub token_program: &'a Address,

    /// Data expected:
    ///
    /// confidential transfer fee authority
    pub authority: Option<&'data Address>,

    /// `ElGamal` public key used to encrypt withheld fees.
    pub withdraw_withheld_authority_elgamal_pubkey: &'data [u8; ELGAMAL_PUBKEY_LEN],
}

impl InitializeConfidentialTransferFeeConfig<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // instruction data
        let mut instruction_data = [UNINIT_BYTE; 2 + 32 + ELGAMAL_PUBKEY_LEN];

        // discrminators
        write_bytes(
            &mut instruction_data[..2],
            &[
                ExtensionDiscriminator::ConfidentialTransferFee as u8,
                Self::DISCRIMINATOR,
            ],
        );

        // confidential transfer fee authority
        match self.authority {
            Some(authority_address) => {
                write_bytes(&mut instruction_data[2..34], authority_address.as_ref())
            }
            None => write_bytes(&mut instruction_data[2..34], &[0u8; 32]),
        };

        // elgamal pubkey to encryp withheld fee
        write_bytes(
            &mut instruction_data[34..34 + ELGAMAL_PUBKEY_LEN],
            self.withdraw_withheld_authority_elgamal_pubkey.as_ref(),
        );

        // instruction
        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &[InstructionAccount::writable(self.mint.address())],
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke(&instruction, &[self.mint])
    }
}
