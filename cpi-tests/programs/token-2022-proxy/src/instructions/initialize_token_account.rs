use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022,
};

pub fn initialize_token_account(accounts: &[AccountInfo]) -> ProgramResult {
    let [token_account, mint, owner, rent_sysvar, token_program] = accounts else {
        Err(ProgramError::InvalidAccountData)?
    };

    pinocchio_token_2022::instructions::InitializeAccount {
        account: token_account,
        mint,
        owner,
        rent_sysvar,
        token_program: token_program.key(),
    }
    .invoke()
}
