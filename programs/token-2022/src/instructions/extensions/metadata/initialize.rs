use {
    super::constants::MAX_IX_DATA,
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

/// Initialize token metadata for a Token-2022 mint.
///
/// This instruction creates and populates the metadata account with
/// the token's name, symbol, and URI.
///
/// ### Accounts:
///   0. `[WRITE]` Metadata account
///   1. `[]` Update authority
///   2. `[]` Mint
///   3. `[SIGNER]` Mint authority
pub struct InitializeMetadata<'a, 'b> {
    /// The metadata account to initialize
    pub metadata: &'a AccountView,
    /// The authority that can update the metadata
    pub update_authority: &'a AccountView,
    /// The mint account
    pub mint: &'a AccountView,
    /// The mint authority (must sign)
    pub mint_authority: &'a AccountView,
    /// Token name
    pub name: &'a str,
    /// Token symbol
    pub symbol: &'a str,
    /// URI to token metadata
    pub uri: &'a str,
    /// Token program (Token-2022).
    pub token_program: &'b Address,
}

impl InitializeMetadata<'_, '_> {
    /// Based on `spl_token_metadata_interface` hash.
    pub const DISCRIMINATOR: [u8; 8] = [210, 225, 30, 162, 88, 184, 77, 141];

    /// Invoke the `InitializeMetadata` instruction.
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    /// Invoke the `InitializeMetadata` instruction with signers.
    ///
    /// Instruction data layout:
    /// - `[0..8]`: instruction discriminator (8 bytes)
    /// - `[8..12]`: name length (4 bytes, `u32`)
    /// - `[12..12+N]`: name string (N bytes, UTF-8)
    /// - `[..+4]`: symbol length (4 bytes, `u32`)
    /// - `[..+S]`: symbol string (S bytes, UTF-8)
    /// - `[..+4]`: uri length (4 bytes, `u32`)
    /// - `[..+U]`: uri string (U bytes, UTF-8)
    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let ix_len = 8 + 4 + self.name.len() + 4 + self.symbol.len() + 4 + self.uri.len();

        if ix_len > MAX_IX_DATA {
            return Err(ProgramError::InvalidArgument);
        }

        let mut ix_data = [UNINIT_BYTE; MAX_IX_DATA];
        let mut offset = 0;

        // Set 8-byte discriminator
        write_bytes(&mut ix_data[offset..offset + 8], &Self::DISCRIMINATOR);
        offset += 8;

        // Set name length and name data bytes
        write_bytes(
            &mut ix_data[offset..offset + 4],
            &(self.name.len() as u32).to_le_bytes(),
        );
        offset += 4;
        write_bytes(
            &mut ix_data[offset..offset + self.name.len()],
            self.name.as_bytes(),
        );
        offset += self.name.len();

        // Set symbol length and symbol data bytes
        write_bytes(
            &mut ix_data[offset..offset + 4],
            &(self.symbol.len() as u32).to_le_bytes(),
        );
        offset += 4;
        write_bytes(
            &mut ix_data[offset..offset + self.symbol.len()],
            self.symbol.as_bytes(),
        );
        offset += self.symbol.len();

        // Set uri length and uri data bytes
        write_bytes(
            &mut ix_data[offset..offset + 4],
            &(self.uri.len() as u32).to_le_bytes(),
        );
        offset += 4;
        write_bytes(
            &mut ix_data[offset..offset + self.uri.len()],
            self.uri.as_bytes(),
        );

        let instruction_accounts: [InstructionAccount; 4] = [
            InstructionAccount::writable(self.metadata.address()),
            InstructionAccount::readonly(self.update_authority.address()),
            InstructionAccount::readonly(self.mint.address()),
            InstructionAccount::readonly_signer(self.mint_authority.address()),
        ];

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &instruction_accounts,
            data: unsafe { from_raw_parts(ix_data.as_ptr() as _, ix_len) },
        };

        invoke_signed(
            &instruction,
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
