use {
    crate::instructions::extensions::{
        transfer_hook::state::{
            offset_transfer_hook_initialize as OFFSET, TransferHookInstruction,
        },
        ExtensionDiscriminator,
    },
    pinocchio::{
        account_info::AccountInfo,
        cpi::invoke_signed,
        instruction::{AccountMeta, Instruction, Signer},
        pubkey::Pubkey,
        ProgramResult,
    },
};

pub struct InitializeTransferHook<'a, 'b> {
    /// Mint Account to initialize.
    pub mint_account: &'a AccountInfo,
    /// Optional authority that can set the transfer hook program id
    pub authority: Option<&'b Pubkey>,
    /// Program that authorizes the transfer
    pub program_id: Option<&'b Pubkey>,
    /// Token Program
    pub token_program: &'b Pubkey,
}

impl InitializeTransferHook<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [AccountMeta::writable(self.mint_account.key())];

        let mut buffer = [0u8; 66];
        let data = initialize_instruction_data(&mut buffer, self.authority, self.program_id);

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data,
        };

        invoke_signed(&instruction, &[self.mint_account], signers)
    }
}

#[inline(always)]
fn initialize_instruction_data<'a>(
    buffer: &'a mut [u8],
    authority: Option<&Pubkey>,
    program_id: Option<&Pubkey>,
) -> &'a [u8] {
    let mut offset = OFFSET::START as usize;

    // Encode discriminators (TransferHook + Initialize)
    buffer[..offset].copy_from_slice(&[
        ExtensionDiscriminator::TransferHook as u8,
        TransferHookInstruction::Initialize as u8,
    ]);

    // Set authority at offset [2..34]
    if let Some(x) = authority {
        buffer[offset..offset + OFFSET::AUTHORITY_PUBKEY as usize].copy_from_slice(x);
    } else {
        buffer[offset..offset + OFFSET::AUTHORITY_PUBKEY as usize].copy_from_slice(&[0; 32]);
    }

    // shift offset past authority pubkey
    offset += OFFSET::AUTHORITY_PUBKEY as usize;

    // Set program_id at offset [34..66]
    if let Some(x) = program_id {
        buffer[offset..OFFSET::END as usize].copy_from_slice(x);
    } else {
        buffer[offset..OFFSET::END as usize].copy_from_slice(&[0; 32]);
    }

    buffer
}
