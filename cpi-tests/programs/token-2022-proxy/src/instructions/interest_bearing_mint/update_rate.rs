use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022,
    spl_token_2022_interface::instruction::decode_instruction_data,
};

pub fn update_rate(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
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

    let rate: i16 = *decode_instruction_data(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    pinocchio_token_2022::extension::interest_bearing_mint::UpdateRate {
        mint,
        authority,
        rate,
        signers,
        token_program: token_program.key(),
    }
    .invoke()
}
