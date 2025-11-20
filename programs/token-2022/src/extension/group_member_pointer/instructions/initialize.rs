use {
    crate::extension::{
        consts::ExtensionDiscriminator,
        group_member_pointer::state::{
            offset_group_member_pointer_initialize as OFFSET,
            InstructionDiscriminatorGroupMemberPointer,
        },
    },
    pinocchio::{
        account_info::AccountInfo,
        cpi::invoke_signed,
        instruction::{AccountMeta, Instruction, Signer},
        pubkey::Pubkey,
        ProgramResult,
    },
};

/// Initialize a new mint with a group member pointer
///
/// Accounts expected by this instruction:
///
///  0. `[writable]` The mint to initialize.
pub struct Initialize<'a> {
    /// Mint Account
    pub mint: &'a AccountInfo,
    /// Optional authority that can set the member address
    pub authority: Option<&'a Pubkey>,
    /// Optional account address that holds the member
    pub member_address: Option<&'a Pubkey>,
    /// Token Program
    pub token_program: &'a Pubkey,
}

impl Initialize<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [AccountMeta::writable(self.mint.key())];

        let mut buffer = [0u8; OFFSET::END as usize];
        let data = initialize_instruction_data(&mut buffer, self.authority, self.member_address);

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data,
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}

pub fn initialize_instruction_data<'a>(
    buffer: &'a mut [u8],
    authority: Option<&'a Pubkey>,
    member_address: Option<&'a Pubkey>,
) -> &'a [u8] {
    let mut offset = OFFSET::START as usize;

    // Set discriminators
    buffer[..offset].copy_from_slice(&[
        ExtensionDiscriminator::GroupMemberPointer as u8,
        InstructionDiscriminatorGroupMemberPointer::Initialize as u8,
    ]);

    // Set authority
    if let Some(x) = authority {
        buffer[offset..offset + OFFSET::AUTHORITY_PUBKEY as usize].copy_from_slice(x);
    }
    offset += OFFSET::AUTHORITY_PUBKEY as usize;

    // Set member_address
    if let Some(x) = member_address {
        buffer[offset..offset + OFFSET::MEMBER_ADDRESS_PUBKEY as usize].copy_from_slice(x);
    }

    buffer
}
