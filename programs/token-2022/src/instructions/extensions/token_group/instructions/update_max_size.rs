use {
    crate::instructions::extensions::token_group::state::{
        offset_token_group_update_max_size as OFFSET, InstructionDiscriminatorTokenGroup,
    },
    pinocchio::{
        account_info::AccountInfo,
        cpi::invoke_signed,
        instruction::{AccountMeta, Instruction, Signer},
        pubkey::Pubkey,
        ProgramResult,
    },
};

/// Update the max size of a `Group`
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` Group
///   1. `[signer]` Update authority
pub struct UpdateGroupMaxSize<'a, 'b> {
    /// Group Account
    pub group: &'a AccountInfo,
    // /// Update authority
    pub update_authority: &'a AccountInfo,
    /// New max size for the group
    pub max_size: u64,
    /// Token Group Program
    pub program_id: &'b Pubkey,
}

impl UpdateGroupMaxSize<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.group.key()),
            AccountMeta::readonly_signer(self.update_authority.key()),
        ];

        let mut buffer = [0u8; OFFSET::END as usize];
        let data = update_group_max_size_instruction_data(&mut buffer, self.max_size);

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: &account_metas,
            data,
        };

        invoke_signed(&instruction, &[self.group, self.update_authority], signers)
    }
}

#[inline(always)]
fn update_group_max_size_instruction_data<'a>(buffer: &'a mut [u8], max_size: u64) -> &'a [u8] {
    let offset = OFFSET::START as usize;

    // Set discriminators
    buffer[..offset].copy_from_slice(
        &(InstructionDiscriminatorTokenGroup::UpdateGroupMaxSize as u64).to_le_bytes(),
    );

    // Set max_size
    buffer[offset..offset + OFFSET::MAX_SIZE as usize].copy_from_slice(&max_size.to_le_bytes());

    buffer
}
