use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022::extension::cpi_guard,
};

pub fn disable_guard(accounts: &[AccountInfo], _instruction_data: &[u8]) -> ProgramResult {
    // The accounts should be: [account, owner, ...signers, token_program]

    if accounts.len() < 4 {
        Err(ProgramError::NotEnoughAccountKeys)?
    }

    let token_program = accounts.last().unwrap(); // token_program is always last
    let account = &accounts[0];
    let owner = &accounts[1];
    let signers = &accounts[2..accounts.len() - 1]; // everything between owner and token_program

    cpi_guard::DisableCpiGuard {
        token_account: account,
        owner,
        signers,
        token_program: &token_program.key(),
    }
    .invoke()
}
