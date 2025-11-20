use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022::extension::transfer_hook,
    spl_token_2022_interface::instruction::decode_instruction_data,
};

pub fn update(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    // accounts should be: [mint, authority, ...signers, token_program]

    if accounts.len() < 4 {
        Err(ProgramError::NotEnoughAccountKeys)?
    }

    let token_program = accounts.last().unwrap(); // token_program is always last
    let mint = &accounts[0];
    let authority = &accounts[1];
    let signers = &accounts[2..accounts.len() - 1]; // everything between authority and token_program

    let &spl_token_2022_interface::extension::transfer_hook::instruction::UpdateInstructionData {
        program_id,
    } = decode_instruction_data(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    transfer_hook::UpdateTransferHook {
        mint_account: mint,
        authority,
        signers,
        program_id: Option::<solana_address::Address>::from(program_id)
            .map(|x| x.to_bytes())
            .as_ref(),
        token_program: token_program.key(),
    }
    .invoke()
}

