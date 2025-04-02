use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

use super::get_extension_from_bytes;

/// State of the metadata pointer
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetadataPointer {
    /// Authority that can set the metadata address
    pub authority: Pubkey,
    /// Account address that holds the metadata
    pub metadata_address: Pubkey,
}

impl super::Extension for MetadataPointer {
    const TYPE: super::ExtensionType = super::ExtensionType::MetadataPointer;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl MetadataPointer {
    /// The length of the `MetadataPointer` account data.
    pub const LEN: usize = core::mem::size_of::<MetadataPointer>();

    /// Return a `MetadataPointer` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<MetadataPointer, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let acc_data_bytes = account_info.try_borrow_data()?;
        let acc_data_bytes = acc_data_bytes.as_ref();

        get_extension_from_bytes::<Self>(acc_data_bytes).ok_or(ProgramError::InvalidAccountData)
    }
}

// Instructions
pub struct Initialize<'a> {
    /// The mint that this metadata pointer is associated with
    pub mint: &'a AccountInfo,
    /// The public key for the account that can update the metadata address
    pub authority: Option<Pubkey>,
    /// The account address that holds the metadata
    pub metadata_address: Option<Pubkey>,
}

impl Initialize<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data layout:
        // -  [0] u8: instruction discriminator
        // -  [1] u8: extension instruction discriminator
        // -  [2..34] u8: authority
        // -  [34..66] u8: metadata_address
        let mut instruction_data = [UNINIT_BYTE; 66];
        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[39]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[0]);
        // Set authority as u8 at offset [2..34]
        if let Some(authority) = self.authority {
            write_bytes(&mut instruction_data[2..34], &authority);
        } else {
            write_bytes(&mut instruction_data[2..34], &Pubkey::default());
        }
        // Set metadata_address as u8 at offset [34..66]
        if let Some(metadata_address) = self.metadata_address {
            write_bytes(&mut instruction_data[34..66], &metadata_address);
        } else {
            write_bytes(&mut instruction_data[34..66], &Pubkey::default());
        }

        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 66) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}

pub struct Update<'a> {
    /// The mint that this metadata pointer is associated with
    pub mint: &'a AccountInfo,
    /// The metadata pointer authority
    pub authority: &'a AccountInfo,
    /// The new account address that holds the metadata
    pub new_metadata_address: Option<Pubkey>,
}

impl Update<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data layout:
        // -  [0] u8: instruction discriminator
        // -  [1] u8: extension instruction discriminator
        // -  [2..34] u8: metadata_address
        let mut instruction_data = [UNINIT_BYTE; 34];
        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[39]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[1]);
        // Set metadata_address as u8 at offset [2..34]
        if let Some(new_metadata_address) = self.new_metadata_address {
            write_bytes(&mut instruction_data[2..34], &new_metadata_address);
        } else {
            write_bytes(&mut instruction_data[2..34], &Pubkey::default());
        }

        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 34) },
        };

        invoke_signed(&instruction, &[self.mint, self.authority], signers)
    }
}
