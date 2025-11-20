use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022::extension::cpi_guard,
};

pub fn enable_guard(accounts: &[AccountInfo], _instruction_data: &[u8]) -> ProgramResult {
    // The accounts should be: [account, owner, ...signers, token_program]

    if accounts.len() < 4 {
        Err(ProgramError::NotEnoughAccountKeys)?
    }

    let token_program = accounts.last().unwrap(); // token_program is always last
    let account = &accounts[0];
    let owner = &accounts[1];
    let signers = &accounts[2..accounts.len() - 1]; // everything between owner and token_program

    {
        let acc = account.try_borrow_data()?;
        let state = pinocchio_token_2022::extension::cpi_guard::state::CpiGuard::from_bytes(&acc)?;

        if state.lock_cpi() {
            return Ok(());
        }
    }

    cpi_guard::EnableCpiGuard {
        token_account: account,
        owner,
        signers,
        token_program: &token_program.key(),
    }
    .invoke()
}
