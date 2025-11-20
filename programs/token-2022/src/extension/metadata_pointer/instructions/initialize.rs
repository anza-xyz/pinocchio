use {
    crate::extension::{
        consts::ExtensionDiscriminator,
        metadata_pointer::state::{
            offset_metadata_pointer_initialize as OFFSET, InstructionDiscriminatorMetadataPointer,
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

/// Initialize a new mint with a metadata pointer
///
/// Accounts expected by this instruction:
///
///  0. `[writable]` The mint to initialize.
pub struct Initialize<'a> {
    /// The mint to initialize with the metadata pointer extension.
    pub mint: &'a AccountInfo,
    /// Optional authority that can later update the metadata address.
    pub authority: Option<&'a Pubkey>,
    /// Optional initial metadata address.
    pub metadata_address: Option<&'a Pubkey>,
    /// Token program (Token-2022).
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
        let data = initialize_instruction_data(&mut buffer, self.authority, self.metadata_address);

        let instruction = Instruction {
            program_id: self.token_program,
            data,
            accounts: &account_metas,
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}

pub fn initialize_instruction_data<'a>(
    buffer: &'a mut [u8],
    authority: Option<&'a Pubkey>,
    metadata_address: Option<&'a Pubkey>,
) -> &'a [u8] {
    let mut offset = OFFSET::START as usize;

    // Encode discriminators (Metadata + Initialize)
    buffer[..offset].copy_from_slice(&[
        ExtensionDiscriminator::MetadataPointer as u8,
        InstructionDiscriminatorMetadataPointer::Initialize as u8,
    ]);

    // Write authority pubkey bytes
    if let Some(authority) = authority {
        buffer[offset..offset + OFFSET::AUTHORITY_PUBKEY as usize].copy_from_slice(authority);
    }

    // shift offset past authority pubkey
    offset += OFFSET::AUTHORITY_PUBKEY as usize;

    // Write metadata_address pubkey bytes
    if let Some(metadata_address) = metadata_address {
        buffer[offset..offset + OFFSET::METADATA_ADDRESS_PUBKEY as usize]
            .copy_from_slice(metadata_address);
    }

    buffer
}
