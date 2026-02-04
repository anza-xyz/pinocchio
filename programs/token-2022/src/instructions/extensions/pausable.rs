//! Pausable extension

use crate::instructions::extensions::ExtensionDiscriminator;
use crate::{write_bytes, UNINIT_BYTE};
use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::cpi::Signer;
use solana_instruction_view::{cpi::invoke_signed, InstructionAccount, InstructionView};
use solana_program_error::ProgramResult;

/// Initialize the pausable extension for a mint.
///
/// ### Accounts:
///   0. `[WRITE]` The mint to initialize.
pub struct InitializePausable<'a, 'b> {
    /// The mint to initialize the pausable config
    pub mint: &'a AccountView,
    /// The public key for the account that can pause or resume activity on the mint
    pub authority: &'a Address,
    /// Token Program
    pub token_program: &'b Address,
}

impl InitializePausable<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let instruction_accounts = [InstructionAccount::writable(self.mint.address())];

        // Instruction data layout:
        // -  [0]: token instruction discriminator (PausableExtension)
        // -  [1]: extension sub-instruction (Initialize = 0)
        // -  [2..34]: authority pubkey (32 bytes)

        let mut instruction_data = [UNINIT_BYTE; 34];

        write_bytes(
            &mut instruction_data[0..1],
            &[ExtensionDiscriminator::Pausable as u8],
        );
        write_bytes(&mut instruction_data[1..2], &[Self::DISCRIMINATOR]);
        write_bytes(&mut instruction_data[2..34], &self.authority.to_bytes());

        let instruction = InstructionView {
            program_id: &self.token_program,
            accounts: &instruction_accounts,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 34) },
        };

        invoke_signed(&instruction, &[self.mint], signers)?;

        Ok(())
    }
}

/// Pause a mint, preventing all token operations.
///
/// ### Accounts:
///   0. `[WRITE]` The mint to pause.
///   1. `[SIGNER]` The mint's pause authority.
pub struct Pause<'a, 'b> {
    /// The mint to pause
    pub mint: &'a AccountView,
    /// The mint's pause authority
    pub pause_authority: &'a AccountView,
    /// Token Program
    pub token_program: &'b Address,
}

impl Pause<'_, '_> {
    pub const DISCRIMINATOR: u8 = 1;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let instruction_accounts: [InstructionAccount; 2] = [
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::readonly_signer(self.pause_authority.address()),
        ];

        // Instruction data layout:
        // -  [0]: token instruction discriminator (PausableExtension)
        // -  [1]: extension sub-instruction (Pause = 1)
        let instruction_data = [ExtensionDiscriminator::Pausable as u8, Self::DISCRIMINATOR];

        let instruction = InstructionView {
            program_id: &self.token_program,
            accounts: &instruction_accounts,
            data: &instruction_data,
        };

        invoke_signed(&instruction, &[self.mint, self.pause_authority], signers)?;

        Ok(())
    }
}

/// Resume a paused mint, allowing token operations again.
///
/// ### Accounts:
///   0. `[WRITE]` The mint to resume.
///   1. `[SIGNER]` The mint's pause authority.
pub struct Resume<'a, 'b> {
    /// The mint to resume
    pub mint: &'a AccountView,
    /// The mint's pause authority
    pub pause_authority: &'a AccountView,
    /// Token Program
    pub token_program: &'b Address,
}

impl Resume<'_, '_> {
    pub const DISCRIMINATOR: u8 = 2;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let instruction_accounts: [InstructionAccount; 2] = [
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::readonly_signer(self.pause_authority.address()),
        ];

        // Instruction data layout:
        // -  [0]: token instruction discriminator (PausableExtension)
        // -  [1]: extension sub-instruction (Resume = 2)
        let instruction_data = [ExtensionDiscriminator::Pausable as u8, Self::DISCRIMINATOR];

        let instruction = InstructionView {
            program_id: &self.token_program,
            accounts: &instruction_accounts,
            data: &instruction_data,
        };

        invoke_signed(&instruction, &[self.mint, self.pause_authority], signers)?;

        Ok(())
    }
}
