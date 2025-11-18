use {
    crate::instructions::extensions::token_group::state::{
        offset_token_group_update_authority as OFFSET, InstructionDiscriminatorTokenGroup,
    },
    pinocchio::{
        account_info::AccountInfo,
        cpi::invoke_signed,
        instruction::{AccountMeta, Instruction, Signer},
        pubkey::Pubkey,
        ProgramResult,
    },
};

/// Update the authority of a `Group`
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` Group
///   1. `[signer]` Current update authority
pub struct UpdateGroupAuthority<'a, 'b> {
    /// Group Account
    pub group: &'a AccountInfo,
    /// Current update authority
    pub current_authority: &'a AccountInfo,
    /// New authority for the group, or None to unset
    pub new_authority: Option<&'b Pubkey>,
    /// Token Group Program
    pub program_id: &'b Pubkey,
}

impl UpdateGroupAuthority<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.group.key()),
            AccountMeta::readonly_signer(self.current_authority.key()),
        ];

        let mut buffer = [0u8; OFFSET::END as usize];
        let data = update_group_authority_instruction_data(&mut buffer, self.new_authority);

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: &account_metas,
            data,
        };

        invoke_signed(&instruction, &[self.group, self.current_authority], signers)
    }
}

#[inline(always)]
fn update_group_authority_instruction_data<'a>(
    buffer: &'a mut [u8],
    new_authority: Option<&Pubkey>,
) -> &'a [u8] {
    let offset = OFFSET::START as usize;

    // Set discriminators
    buffer[..offset].copy_from_slice(
        &(InstructionDiscriminatorTokenGroup::UpdateGroupAuthority as u64).to_le_bytes(),
    );

    // Set new_authority (optional)
    if let Some(authority) = new_authority {
        buffer[offset..offset + OFFSET::NEW_AUTHORITY as usize].copy_from_slice(authority.as_ref());
    }

    buffer
}
