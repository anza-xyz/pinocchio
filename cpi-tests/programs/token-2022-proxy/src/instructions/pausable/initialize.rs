use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022::extension::pausable,
    spl_token_2022_interface::instruction::decode_instruction_data,
};

pub fn initialize(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let [mint, token_program] = accounts else {
        Err(ProgramError::InvalidAccountData)?
    };

    let &spl_token_2022_interface::extension::pausable::instruction::InitializeInstructionData {
        authority,
    } = decode_instruction_data(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // Store authority bytes in a local variable to avoid referencing temporaries
    let auth_bytes = authority.to_bytes();

    if pinocchio_token_2022::state::Mint::from_account_info(mint)?.is_initialized() {
        let pausable_config = pinocchio_token_2022::extension::pausable::state::PausableConfig::from_account_info(mint)?;

        if pausable_config.authority() != &auth_bytes {
            Err(ProgramError::InvalidAccountData)?
        }

        return Ok(());
    }

    pausable::InitializePausable {
        mint_account: mint,
        authority: auth_bytes,
        token_program: &token_program.key(),
    }
    .invoke()
}
