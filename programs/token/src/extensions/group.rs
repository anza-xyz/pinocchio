use pinocchio::{account_info::AccountInfo, cpi::invoke_signed, instruction::{AccountMeta, Instruction, Signer}, program_error::ProgramError, pubkey::Pubkey, ProgramResult};

use crate::{write_bytes, TOKEN_2022_PROGRAM_ID, UNINIT_BYTE};

use super::{get_extension_from_bytes, BaseState, Extension, ExtensionType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenGroup {
    /// The authority that can sign to update the group
    /// NOTE: Default Pubkey is equivalent to None.
    pub update_authority: Pubkey,
    /// The associated mint, used to counter spoofing to be sure that group
    /// belongs to a particular mint
    pub mint: Pubkey,
    /// The current number of group members
    pub size: [u8; 8],
    /// The maximum number of group members
    pub max_size: [u8; 8],
}

impl TokenGroup {
    /// The length of the `TokenGroup` account data inlcuding the discriminator.
    pub const LEN: usize = core::mem::size_of::<TokenGroup>() + 8;

    /// Return a `TokenGroup` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<TokenGroup, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let acc_data_bytes = account_info.try_borrow_data()?;
        let acc_data_bytes = acc_data_bytes.as_ref();

        get_extension_from_bytes::<Self>(acc_data_bytes).ok_or(ProgramError::InvalidAccountData)
    }
}

impl Extension for TokenGroup {
    const TYPE: ExtensionType = ExtensionType::TokenGroup;
    const LEN: usize = Self::LEN;
    const BASE_STATE: BaseState = BaseState::Mint;
}

/// Instructions

pub struct InitializeGroup<'a> {
    /// The group to be initialized
    pub group: &'a AccountInfo,
    /// The mint that this group will be associated with
    pub mint: &'a AccountInfo,
    /// The public key for the account that controls the mint
    pub mint_authority: &'a AccountInfo,
    /// The public key for the account that can update the group
    pub update_authority: Option<Pubkey>,
    /// The maximum number of group members
    pub max_size: u64,
}

impl<'a> InitializeGroup<'a> {
    const LEN: usize = 42;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data layout:
        // -  [0] u8: instruction discriminator
        // -  [1] u8: extension instruction discriminator
        // -  [2..34] u8: update_authority
        // -  [34..42] u8: max_size
        let mut instruction_data = [UNINIT_BYTE; Self::LEN];
        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[40]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[0]);
        // Set update_authority as u8 at offset [2..34]
        if let Some(update_authority) = self.update_authority {
            write_bytes(&mut instruction_data[2..34], &update_authority);
        } else {
            write_bytes(&mut instruction_data[2..34], &Pubkey::default());
        }
        // Set max_size as u8 at offset [34..42]
        write_bytes(&mut instruction_data[34..42], &self.max_size.to_le_bytes());

        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.group.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly_signer(self.mint_authority.key()),
        ];

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, Self::LEN) },
        };

        invoke_signed(
            &instruction,
            &[self.group, self.mint, self.mint_authority],
            signers,
        )
    }
}

pub struct UpdateGroupMaxSize<'a> {
    /// The group to be initialized
    pub group: &'a AccountInfo,
    /// The public key for the account that can update the group
    pub update_authority: &'a AccountInfo,
    /// The maximum number of group members
    pub max_size: u64,
}

impl<'a> UpdateGroupMaxSize<'a> {
    const LEN: usize = 10;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data layout:
        // -  [0] u8: instruction discriminator
        // -  [1] u8: extension instruction discriminator
        // -  [2..10] u8: max_size
        let mut instruction_data = [UNINIT_BYTE; Self::LEN];
        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[40]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[1]);
        // Set max_size as u8 at offset [2..10]
        write_bytes(&mut instruction_data[2..10], &self.max_size.to_le_bytes());
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.group.key()),
            AccountMeta::readonly_signer(self.update_authority.key()),
        ];

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, Self::LEN) },
        };

        invoke_signed(&instruction, &[self.group, self.update_authority], signers)
    }
}

// TODO UpdateGroupAuthority
