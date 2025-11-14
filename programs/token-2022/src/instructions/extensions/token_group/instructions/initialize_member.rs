use {
    crate::instructions::extensions::token_group::state::{
        offset_token_group_initialize_member as OFFSET, InstructionDiscriminatorTokenGroup,
    },
    pinocchio::{
        account_info::AccountInfo,
        cpi::invoke_signed,
        instruction::{AccountMeta, Instruction, Signer},
        pubkey::Pubkey,
        ProgramResult,
    },
};

/// Initialize a new `Member` of a `Group`
///
/// Assumes the `Group` has already been initialized,
/// as well as the mint for the member.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` Member
///   1. `[]` Member mint
///   2. `[signer]` Member mint authority
///   3. `[writable]` Group
///   4. `[signer]` Group update authority
pub struct InitializeMember<'a, 'b> {
    /// Member Account
    pub member: &'a AccountInfo,
    /// Member mint
    pub member_mint: &'a AccountInfo,
    /// Member mint authority
    pub member_mint_authority: &'a AccountInfo,
    /// Group Account
    pub group: &'a AccountInfo,
    /// Group update authority
    pub group_update_authority: &'a AccountInfo,
    /// Token Group Program
    pub program_id: &'b Pubkey,
}

impl InitializeMember<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.member.key()),
            AccountMeta::readonly(self.member_mint.key()),
            AccountMeta::readonly_signer(self.member_mint_authority.key()),
            AccountMeta::writable(self.group.key()),
            AccountMeta::readonly_signer(self.group_update_authority.key()),
        ];

        let mut buffer = [0u8; OFFSET::END as usize];
        let data = initialize_member_instruction_data(&mut buffer);

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: &account_metas,
            data,
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

#[inline(always)]
fn initialize_member_instruction_data<'a>(buffer: &'a mut [u8]) -> &'a [u8] {
    let offset = OFFSET::START as usize;

    // Set discriminators
    buffer[..offset].copy_from_slice(
        &(InstructionDiscriminatorTokenGroup::InitializeMember as u64).to_le_bytes(),
    );

    buffer
}
