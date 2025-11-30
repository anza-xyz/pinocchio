use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022,
    spl_token_2022_interface::instruction::decode_instruction_data,
};

pub fn update_multiplier(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    // The accounts should be: [mint, authority, ...signers, token_program]
    // For single authority: [mint, authority, authority (as signer), token_program]
    // For multisig: [mint, authority, signer1, signer2, ..., token_program]

    if accounts.len() < 4 {
        Err(ProgramError::NotEnoughAccountKeys)?;
    }

    let token_program = accounts.last().unwrap(); // token_program is always last
    let mint = &accounts[0];
    let authority = &accounts[1];
    let signers = &accounts[2..accounts.len() - 1]; // everything between authority and token_program

    let &spl_token_2022_interface::extension::scaled_ui_amount::instruction::UpdateMultiplierInstructionData {
        multiplier,
        effective_timestamp,
    } = decode_instruction_data(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    pinocchio_token_2022::extension::scaled_ui_amount::instructions::UpdateMultiplier {
        mint_account: mint,
        authority,
        signers,
        token_program: token_program.key(),
        multiplier: multiplier.into(),
        effective_timestamp: effective_timestamp.into(),
    }
    .invoke()
}
