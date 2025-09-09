#![allow(unexpected_cfgs)]

use {
    pinocchio::{
        account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
        ProgramResult,
    },
    pinocchio_pubkey::declare_id,
    spl_token_2022_interface::instruction::TokenInstruction,
};

pub mod helpers;
mod instructions;

use instructions as i;

entrypoint!(process_instruction);
declare_id!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT");

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let token_instruction = TokenInstruction::unpack(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match token_instruction {
        TokenInstruction::InitializeMint {
            decimals,
            mint_authority,
            freeze_authority,
        } => i::initialize_mint(accounts, decimals, mint_authority, freeze_authority),
        _ => Err(ProgramError::InvalidInstructionData)?,
    }
}
