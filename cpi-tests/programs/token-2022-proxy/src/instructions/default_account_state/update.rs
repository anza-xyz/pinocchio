use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022::extension::default_account_state::instructions::update::UpdateDefaultAccountState,
    spl_token_2022_interface::extension::default_account_state::instruction::decode_instruction,
};

pub fn update(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    // The accounts should be: [mint, freeze_authority, ...signers, token_program]
    // For single authority: [mint, freeze_authority, freeze_authority (as signer), token_program]
    // For multisig: [mint, freeze_authority, signer1, signer2, ..., token_program]

    if accounts.len() < 4 {
        Err(ProgramError::NotEnoughAccountKeys)?
    }

    let token_program = accounts.last().unwrap(); // token_program is always last
    let mint = &accounts[0];
    let freeze_authority = &accounts[1];
    let signers: Vec<&AccountInfo> = accounts[2..accounts.len() - 1].iter().collect(); // everything between freeze_authority and token_program

    let (instruction_type, state) = decode_instruction(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    UpdateDefaultAccountState {
        mint_account: mint,
        freeze_authority,
        state: state as u8,
        signers: &signers,
        token_program: &token_program.key(),
    }
    .invoke()
}
