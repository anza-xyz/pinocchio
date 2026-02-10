extern crate alloc;

use alloc::vec::Vec;
use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    InstructionAccount, InstructionView,
};
use solana_program_error::ProgramResult;

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
pub struct InitializeTokenMetadata<'a, 'b> {
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

impl InitializeTokenMetadata<'_, '_> {
    /// Based on spl_token_metadata_interface hash
    pub const DISCRIMINATOR: [u8; 8] = [210, 225, 30, 162, 88, 184, 77, 141];

    /// Invoke the InitializeTokenMetadata instruction
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    /// Invoke the InitializeTokenMetadata instruction with signers
    ///
    /// Instruction data layout:
    /// - [0..8]: instruction discriminator (8 bytes)
    /// - [8..12]: name length (4 bytes, u32)
    /// - [12..12+`n1`]: name string (`n1` bytes, UTF-8)
    /// - [...]: symbol length (4 bytes, u32)
    /// - [...]: symbol string (`n2` bytes, UTF-8)
    /// - [...]: uri length (4 bytes, u32)
    /// - [...]: uri string (`n3` bytes, UTF-8)
    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let ix_len = 8 // instruction discriminator
                + 4 // name length
                + self.name.len()
                + 4 // symbol length
                + self.symbol.len()
                + 4 // uri length
                + self.uri.len();
        let mut ix_data: Vec<u8> = Vec::with_capacity(ix_len);

        ix_data.extend(Self::DISCRIMINATOR);

        // Set name length and name data bytes
        let name_len = self.name.len() as u32;
        ix_data.extend(&name_len.to_le_bytes());
        ix_data.extend(self.name.as_bytes());

        // Set symbol length and symbol data bytes
        let symbol_len = self.symbol.len() as u32;
        ix_data.extend(&symbol_len.to_le_bytes());
        ix_data.extend(self.symbol.as_bytes());

        // Set uri length and uri data bytes
        let uri_len = self.uri.len() as u32;
        ix_data.extend(&uri_len.to_le_bytes());
        ix_data.extend(self.uri.as_bytes());

        let instruction_accounts: [InstructionAccount; 4] = [
            InstructionAccount::writable(self.metadata.address()),
            InstructionAccount::readonly(self.update_authority.address()),
            InstructionAccount::readonly(self.mint.address()),
            InstructionAccount::readonly_signer(self.mint_authority.address()),
        ];

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &instruction_accounts,
            data: &ix_data,
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
