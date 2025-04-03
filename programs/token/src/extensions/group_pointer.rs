use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

use super::{get_extension_from_bytes, BaseState, Extension, ExtensionType};

/// State of the token group pointer
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroupPointer {
    /// Authority that can set the group address
    pub authority: Pubkey,
    /// Account address that holds the group
    pub group_address: Pubkey,
}

impl GroupPointer {
    /// The length of the `GroupPointer` account data.
    pub const LEN: usize = core::mem::size_of::<GroupPointer>();

    /// Return a `GroupPointer` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<GroupPointer, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let acc_data_bytes = account_info.try_borrow_data()?;
        let acc_data_bytes = acc_data_bytes.as_ref();

        get_extension_from_bytes::<Self>(acc_data_bytes).ok_or(ProgramError::InvalidAccountData)
    }
}

impl Extension for GroupPointer {
    const TYPE: ExtensionType = ExtensionType::GroupPointer;
    const LEN: usize = Self::LEN;
    const BASE_STATE: BaseState = BaseState::Mint;
}

pub struct Initialize<'a> {
    /// Mint of the group pointer
    pub mint: &'a AccountInfo,
    /// The public key for the account that can update the group address
    pub authority: Option<Pubkey>,
    /// The account address that holds the group
    pub group_address: Option<Pubkey>,
}

impl Initialize<'_> {
    const LEN: usize = 66;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data layout:
        // -  [0] u8: instruction discriminator
        // -  [1] u8: extension instruction discriminator
        // -  [2..34] u8: authority
        // -  [34..66] u8: group_address
        let mut instruction_data = [UNINIT_BYTE; Self::LEN];
        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[40]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[0]);
        // Set authority as u8 at offset [2..34]
        if let Some(authority) = self.authority {
            write_bytes(&mut instruction_data[2..34], &authority);
        } else {
            write_bytes(&mut instruction_data[2..34], &Pubkey::default());
        }
        // Set group_address as u8 at offset [34..66]
        if let Some(group_address) = self.group_address {
            write_bytes(&mut instruction_data[34..66], &group_address);
        } else {
            write_bytes(&mut instruction_data[34..66], &Pubkey::default());
        }

        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, Self::LEN) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}

pub struct Update<'a> {
    /// Mint of the group pointer
    pub mint: &'a AccountInfo,
    /// The public key for the account that can update the group address
    pub authority: &'a AccountInfo,
    /// The new account address that holds the group configurations
    pub group_address: Option<Pubkey>,
}

impl Update<'_> {
    const LEN: usize = 34;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data layout:
        // -  [0] u8: instruction discriminator
        // -  [1] u8: extension instruction discriminator
        // -  [2..34] u8: group_address
        let mut instruction_data = [UNINIT_BYTE; Self::LEN];
        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[40]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[1]);
        // Set group_address as u8 at offset [2..34]
        if let Some(group_address) = self.group_address {
            write_bytes(&mut instruction_data[2..34], &group_address);
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
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, Self::LEN) },
        };

        invoke_signed(&instruction, &[self.mint, self.authority], signers)
    }
}
