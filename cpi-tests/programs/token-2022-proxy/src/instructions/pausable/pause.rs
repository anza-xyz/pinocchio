use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022::extension::pausable,
};

pub fn pause(accounts: &[AccountInfo], _instruction_data: &[u8]) -> ProgramResult {
    // accounts should be: [mint, authority, ...signers, token_program]

    if accounts.len() < 4 {
        Err(ProgramError::NotEnoughAccountKeys)?
    }

    let token_program = accounts.last().unwrap(); // token_program is always last
    let mint = &accounts[0];
    let authority = &accounts[1];
    let signers = &accounts[2..accounts.len() - 1]; // everything between authority and token_program

    {
        let acc = mint.try_borrow_data()?;
        let state = pinocchio_token_2022::extension::pausable::state::PausableConfig::from_bytes(&acc)?;

        if state.is_paused() {
            return Ok(());
        }
    }

    pausable::Pause {
        mint_account: mint,
        authority,
        signers,
        token_program: &token_program.key(),
    }
    .invoke()
}
