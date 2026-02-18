use {
    crate::{
        instructions::ExtensionDiscriminator, write_bytes, AE_CIPHERTEXT_LEN, ELGAMAL_PUBKEY_LEN,
        UNINIT_BYTE,
    },
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initializes confidential mints and burns for a mint.
///
/// The `ConfidentialMintBurnInstruction::InitializeMint` instruction
/// requires no signers and MUST be included within the same Transaction
/// as `TokenInstruction::InitializeMint`. Otherwise another party can
/// initialize the configuration.
///
/// The instruction fails if the `TokenInstruction::InitializeMint`
/// instruction has already executed for the mint.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The SPL Token mint.
pub struct InitializeMint<'a, 'data> {
    /// The SPL Token mint
    pub mint: &'a AccountView,
    /// The token program address
    pub token_program: &'a Address,

    /// Data expected:
    ///
    /// The `ElGamal` pubkey used to encrypt the confidential supply
    pub supply_elgamal_pubkey: &'data [u8; ELGAMAL_PUBKEY_LEN],
    /// The initial 0 supply encrypted with the supply `AES` key
    pub decryptable_supply: &'data [u8; AE_CIPHERTEXT_LEN],
}

impl InitializeMint<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let mut instruction_data = [UNINIT_BYTE; 2 + 32 + 36];

        // SAFETY: Allocations are valid to the maximum number of bytes
        unsafe {
            // extension discriminator
            instruction_data
                .get_unchecked_mut(0)
                .write(ExtensionDiscriminator::ConfidentialMintBurn as u8);

            // extension instruction discrminator
            instruction_data
                .get_unchecked_mut(1)
                .write(Self::DISCRIMINATOR);
        }

        // supply elgamal pubkey
        write_bytes(
            &mut instruction_data[2..34],
            self.supply_elgamal_pubkey.as_ref(),
        );

        // decrpt-able supply
        write_bytes(
            &mut instruction_data[34..70],
            self.decryptable_supply.as_ref(),
        );

        invoke(
            &InstructionView {
                program_id: self.token_program,
                data: unsafe {
                    from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len())
                },
                accounts: &[InstructionAccount::writable(self.mint.address())],
            },
            &[self.mint],
        )
    }
}
