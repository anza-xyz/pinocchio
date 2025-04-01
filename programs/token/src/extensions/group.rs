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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenGroup {
    pub _discriminator: [u8; 8],
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
    pub const LEN: usize = core::mem::size_of::<TokenGroup>();

    // Discriminator for the TokenGroup state.
    // const DISCRIMINATOR: [u8; 8] = [214, 15, 63, 132, 49, 119, 209, 40];

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenGroupMember {
    pub _discriminator: [u8; 8],
    /// The associated mint, used to counter spoofing to be sure that member
    /// belongs to a particular mint
    pub mint: Pubkey,
    /// The pubkey of the `TokenGroup`
    pub group: Pubkey,
    /// The member number
    pub member_number: [u8; 8],
}

impl TokenGroupMember {
    /// The length of the `TokenGroupMember` account data inlcuding the discriminator.
    pub const LEN: usize = core::mem::size_of::<TokenGroupMember>() + 8;

    // Discriminator for the TokenGroupMember state
    // const DISCRIMINATOR: [u8; 8] = [254, 50, 168, 134, 88, 126, 100, 186];

    /// Return a `TokenGroupMember` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<TokenGroupMember, ProgramError> {
        if !account_info.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let acc_data_bytes = account_info.try_borrow_data()?;
        let acc_data_bytes = acc_data_bytes.as_ref();

        get_extension_from_bytes::<Self>(acc_data_bytes).ok_or(ProgramError::InvalidAccountData)
    }
}

impl Extension for TokenGroupMember {
    const TYPE: ExtensionType = ExtensionType::TokenGroupMember;
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
    const LEN: usize = 48;
    const DISCRIMINATOR: [u8; 8] = [121, 113, 108, 39, 54, 51, 0, 4];

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data layout:
        // -  [0..8] [u8; 8]: instruction discriminator
        // -  [8..40] Pubkey: update_authority
        // -  [40..48] u64: max_size
        let mut instruction_data = [UNINIT_BYTE; Self::LEN];
        // Set 8-byte discriminator [0..8]
        write_bytes(&mut instruction_data[0..8], &Self::DISCRIMINATOR);
        // Set update_authority as u8 at offset [8..40]
        if let Some(update_authority) = self.update_authority {
            write_bytes(&mut instruction_data[8..40], &update_authority);
        } else {
            write_bytes(&mut instruction_data[8..40], &Pubkey::default());
        }
        // Set max_size as u8 at offset [40..48]
        write_bytes(&mut instruction_data[40..48], &self.max_size.to_le_bytes());

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
    /// The group to be updated
    pub group: &'a AccountInfo,
    /// The public key for the account that can update the group
    pub update_authority: &'a AccountInfo,
    /// The maximum number of group members
    pub max_size: u64,
}

impl<'a> UpdateGroupMaxSize<'a> {
    const LEN: usize = 16;
    const DISCRIMINATOR: [u8; 8] = [108, 37, 171, 143, 248, 30, 18, 110];

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data layout:
        // -  [0..8] [u8; 8]: instruction discriminator
        // -  [8..16] u8: max_size
        let mut instruction_data = [UNINIT_BYTE; Self::LEN];
        // Set 8-byte discriminator [0..8]
        write_bytes(&mut instruction_data[0..8], &Self::DISCRIMINATOR);
        // Set max_size as u8 at offset [8..16]
        write_bytes(&mut instruction_data[8..16], &self.max_size.to_le_bytes());
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

pub struct UpdateGroupAuthority<'a> {
    /// The group to be updated
    pub group: &'a AccountInfo,
    /// The public key for the account that can update the group
    pub current_authority: &'a AccountInfo,
    /// The new authority for the TokenGroup
    pub new_authority: Option<Pubkey>,
}

impl<'a> UpdateGroupAuthority<'a> {
    const LEN: usize = 40;
    const DISCRIMINATOR: [u8; 8] = [161, 105, 88, 1, 237, 221, 216, 203];

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data layout:
        // -  [0..8] [u8; 8]: instruction discriminator
        // -  [8..40] Pubkey: new authority
        let mut instruction_data = [UNINIT_BYTE; Self::LEN];
        // Set 8-byte discriminator [0..8]
        write_bytes(&mut instruction_data[0..8], &Self::DISCRIMINATOR);
        // Set update_authority as u8 at offset [8..40]
        if let Some(update_authority) = self.new_authority {
            write_bytes(&mut instruction_data[8..40], &update_authority);
        } else {
            write_bytes(&mut instruction_data[8..40], &Pubkey::default());
        }
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.group.key()),
            AccountMeta::readonly_signer(self.current_authority.key()),
        ];

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, Self::LEN) },
        };

        invoke_signed(&instruction, &[self.group, self.current_authority], signers)
    }
}

pub struct InitializeMember<'a> {
    /// The group the member belongs to
    pub group: &'a AccountInfo,
    /// Update authority of the group
    pub group_update_authority: &'a AccountInfo,
    /// Member account
    pub member: &'a AccountInfo,
    /// Token Mint of the Member to be added to the group
    pub member_mint: &'a AccountInfo,
    /// Mint authority of the `member_mint`
    pub member_mint_authority: &'a AccountInfo,
}

impl<'a> InitializeMember<'a> {
    const LEN: usize = 8;
    const DISCRIMINATOR: [u8; 8] = [152, 32, 222, 176, 223, 237, 116, 134];

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data layout:
        // -  [0..8] [u8; 8]: instruction discriminator
        let mut instruction_data = [UNINIT_BYTE; Self::LEN];
        // Set 8-byte discriminator [0..8]
        write_bytes(&mut instruction_data[0..8], &Self::DISCRIMINATOR);

        let account_metas: [AccountMeta; 5] = [
            AccountMeta::writable(self.member.key()),
            AccountMeta::readonly(self.member_mint.key()),
            AccountMeta::readonly_signer(self.member_mint_authority.key()),
            AccountMeta::writable(self.group.key()),
            AccountMeta::readonly_signer(self.group_update_authority.key()),
        ];

        let instruction = Instruction {
            program_id: &TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, Self::LEN) },
        };

        invoke_signed(
            &instruction,
            &[
                self.member,
                self.member_mint,
                self.member_mint_authority,
                self.group,
                self.group_update_authority,
            ],
            signers,
        )
    }
}
