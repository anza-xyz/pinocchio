use {
    crate::helpers::from_optional_non_zero_pubkey,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022,
    spl_token_2022_interface::instruction::decode_instruction_data,
};

pub fn initialize(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let [mint, token_program] = accounts else {
        Err(ProgramError::InvalidAccountData)?
    };

    let &spl_token_2022_interface::extension::interest_bearing_mint::instruction::InitializeInstructionData {
        rate_authority,
        rate,
    } = decode_instruction_data(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let rate_authority = from_optional_non_zero_pubkey(rate_authority);

    if pinocchio_token_2022::state::Mint::from_account_info(mint)?.is_initialized() {
        let config =
            pinocchio_token_2022::extension::interest_bearing_mint::state::InterestBearingConfig::from_account_info(
                mint,
            )?;

        if config.rate_authority() != rate_authority.as_ref() {
            Err(ProgramError::InvalidAccountData)?
        }

        return Ok(());
    }

    pinocchio_token_2022::extension::interest_bearing_mint::Initialize {
        mint,
        rate_authority: rate_authority.as_ref(),
        rate: rate.into(),
        token_program: &token_program.key(),
    }
    .invoke()
}
