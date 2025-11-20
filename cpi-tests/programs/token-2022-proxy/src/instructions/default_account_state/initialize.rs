use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022::extension::default_account_state::instructions::initialize::InitializeDefaultAccountState,
    spl_token_2022_interface::extension::default_account_state::instruction::decode_instruction,
};

pub fn initialize(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let [mint, token_program] = accounts else {
        Err(ProgramError::InvalidAccountData)?
    };

    let (_, state) = decode_instruction(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    if pinocchio_token_2022::state::Mint::from_account_info(mint)?.is_initialized() {
        let config = pinocchio_token_2022::extension::default_account_state::state::DefaultAccountStateConfig::from_account_info(mint)?;

        if config.state() != state as u8 {
            Err(ProgramError::InvalidAccountData)?
        }

        return Ok(());
    }

    InitializeDefaultAccountState {
        mint_account: mint,
        state: state as u8,
        token_program: &token_program.key(),
    }
    .invoke()
}
