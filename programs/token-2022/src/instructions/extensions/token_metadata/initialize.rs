use {
    crate::{write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Initializes a `TokenMetadata` TLV entry.
///
/// By the end of the instruction, the metadata account must contain a
/// `TokenMetadata` entry with the given name, symbol and URI.
///
/// Token-2022 only supports storing the metadata in the mint itself, so the
/// metadata account must be the mint. The mint must have the
/// `MetadataPointer` extension pointing to itself and enough lamports to
/// cover the rent of the additional space.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The metadata account.
///   1. `[]` The update authority.
///   2. `[]` The mint.
///   3. `[signer]` The mint authority.
pub struct Initialize<'a, 'b> {
    /// The metadata account.
    pub metadata: &'a AccountView,

    /// The update authority.
    pub update_authority: &'a AccountView,

    /// The mint.
    pub mint: &'a AccountView,

    /// The mint authority.
    pub mint_authority: &'a AccountView,

    /// The longer name of the token.
    pub name: &'b str,

    /// The shortened symbol for the token.
    pub symbol: &'b str,

    /// The URI pointing to richer metadata.
    pub uri: &'b str,

    /// The token program.
    pub token_program: &'b Address,
}

impl Initialize<'_, '_> {
    /// Hash of `spl_token_metadata_interface:initialize_account`.
    pub const DISCRIMINATOR: [u8; 8] = [210, 225, 30, 162, 88, 184, 77, 141];

    /// Maximum instruction data length.
    pub const MAX_DATA_LEN: usize = 1024;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data.

        let expected_data = 8 + 12 + self.name.len() + self.symbol.len() + self.uri.len();

        if expected_data > Self::MAX_DATA_LEN {
            return Err(ProgramError::InvalidArgument);
        }

        let mut instruction_data = [UNINIT_BYTE; Self::MAX_DATA_LEN];

        write_bytes(&mut instruction_data[0..8], &Self::DISCRIMINATOR);

        let mut offset = 8;

        for value in [self.name, self.symbol, self.uri] {
            write_bytes(
                &mut instruction_data[offset..offset + 4],
                &(value.len() as u32).to_le_bytes(),
            );
            write_bytes(
                &mut instruction_data[offset + 4..offset + 4 + value.len()],
                value.as_bytes(),
            );
            offset += 4 + value.len();
        }

        invoke_signed(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[
                    InstructionAccount::writable(self.metadata.address()),
                    InstructionAccount::readonly(self.update_authority.address()),
                    InstructionAccount::readonly(self.mint.address()),
                    InstructionAccount::readonly_signer(self.mint_authority.address()),
                ],
                // SAFETY: instruction data is initialized up to `expected_data`.
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, expected_data) },
            },
            &[
                self.metadata,
                self.update_authority,
                self.mint,
                self.mint_authority,
            ],
            signers,
        )
    }
}
