use {
    crate::instructions::extensions::token_group::state::{
        offset_token_group_initialize_group as OFFSET, InstructionDiscriminatorTokenGroup,
    },
    pinocchio::{
        account_info::AccountInfo,
        cpi::invoke_signed,
        instruction::{AccountMeta, Instruction, Signer},
        pubkey::Pubkey,
        ProgramResult,
    },
};

/// Initialize a new `Group`
///
/// Assumes one has already initialized a mint for the group.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` Group
///   1. `[]` Mint
///   2. `[signer]` Mint authority
pub struct InitializeGroup<'a, 'b> {
    /// Group Account
    pub group: &'a AccountInfo,
    /// Mint Account
    pub mint: &'a AccountInfo,
    /// Mint authority
    pub mint_authority: &'a AccountInfo,
    /// Update authority for the group
    pub update_authority: Option<&'b Pubkey>,
    /// The maximum number of group members
    pub max_size: u64,
    /// Token Group Program
    pub program_id: &'b Pubkey,
}

impl InitializeGroup<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.group.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly_signer(self.mint_authority.key()),
        ];

        let mut buffer = [0u8; OFFSET::END as usize];
        let data =
            initialize_group_instruction_data(&mut buffer, self.update_authority, self.max_size);

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: &account_metas,
            data,
        };

        invoke_signed(
            &instruction,
            &[self.group, self.mint, self.mint_authority],
            signers,
        )
    }
}

#[inline(always)]
fn initialize_group_instruction_data<'a>(
    buffer: &'a mut [u8],
    update_authority: Option<&Pubkey>,
    max_size: u64,
) -> &'a [u8] {
    let mut offset = OFFSET::START as usize;

    // Set discriminators
    buffer[..offset].copy_from_slice(
        &(InstructionDiscriminatorTokenGroup::InitializeGroup as u64).to_le_bytes(),
    );

    // Set update_authority
    if let Some(x) = update_authority {
        buffer[offset..offset + OFFSET::UPDATE_AUTHORITY as usize].copy_from_slice(x);
    }
    offset += OFFSET::UPDATE_AUTHORITY as usize;

    // Set max_size
    buffer[offset..offset + OFFSET::MAX_SIZE as usize].copy_from_slice(&max_size.to_le_bytes());

    buffer
}
