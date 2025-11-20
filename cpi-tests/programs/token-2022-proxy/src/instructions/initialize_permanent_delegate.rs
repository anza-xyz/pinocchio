use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022,
    solana_address::Address,
};

pub fn initialize_permanent_delegate(accounts: &[AccountInfo], delegate: Address) -> ProgramResult {
    let [mint, token_program] = accounts else {
        Err(ProgramError::InvalidAccountData)?
    };

    if pinocchio_token_2022::state::Mint::from_account_info(mint)?.is_initialized() {
        let state = pinocchio_token_2022::extension::permanent_delegate::state::PermanentDelegate::from_account_info(mint)?;

        if state.delegate() != Some(&delegate.to_bytes()) {
            Err(ProgramError::InvalidAccountData)?
        }

        return Ok(());
    }

    pinocchio_token_2022::extension::permanent_delegate::InitializePermanentDelegate {
        mint,
        delegate: delegate.to_bytes(),
        token_program: &token_program.key(),
    }
    .invoke()
}
