use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{self, AccountMeta, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

use super::get_extension_from_bytes;

/// State of the mint close authority
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MintCloseAuthority {
    /// Optional authority to close the mint
    pub close_authority: Pubkey,
}

impl super::Extension for MintCloseAuthority {
    const TYPE: super::ExtensionType = super::ExtensionType::MintCloseAuthority;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl MintCloseAuthority {
    /// The length of the `MintCloseAuthority` account data.
    pub const LEN: usize = core::mem::size_of::<MintCloseAuthority>();

    /// Return a `MintCloseAuthority` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<MintCloseAuthority, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let acc_data_bytes = account_info.try_borrow_data()?;
        let acc_data_bytes = acc_data_bytes.as_ref();

        get_extension_from_bytes::<Self>(acc_data_bytes).ok_or(ProgramError::InvalidAccountData)
    }
}

// Instructions
pub struct InitializeMintCloseAuthority<'a> {
    /// The mint to initialize the close authority
    pub mint: &'a AccountInfo,
    /// The public key for the account that can close the mint
    pub close_authority: Option<Pubkey>,
}

impl<'a> InitializeMintCloseAuthority<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [AccountMeta::writable(self.mint.key())];
        // Instruction data Layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..33]: close authority (32 bytes, Pubkey)

        let mut instruction_data = [UNINIT_BYTE; 33];
        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[25]);
        // Set close authority as Pubkey at offset [1..33]
        if let Some(close_authority) = self.close_authority {
            write_bytes(&mut instruction_data[1..33], &close_authority);
        } else {
            write_bytes(&mut instruction_data[1..33], &Pubkey::default());
        }

        let instruction = instruction::Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 33) },
        };

        invoke_signed(&instruction, &[self.mint], signers)?;

        Ok(())
    }
}
