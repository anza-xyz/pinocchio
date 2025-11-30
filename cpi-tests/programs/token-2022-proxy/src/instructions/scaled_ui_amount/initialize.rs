use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022::extension::scaled_ui_amount,
    crate::helpers::from_optional_non_zero_pubkey,
    spl_token_2022_interface::instruction::decode_instruction_data,
};

pub fn initialize_scaled_ui_amount(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let [mint, token_program] = accounts else {
        Err(ProgramError::InvalidAccountData)?
    };

    let &spl_token_2022_interface::extension::scaled_ui_amount::instruction::InitializeInstructionData {
        authority,
        multiplier,
    } = decode_instruction_data(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let authority = from_optional_non_zero_pubkey(authority)
        .unwrap_or_else(pinocchio::pubkey::Pubkey::default);

    if pinocchio_token_2022::state::Mint::from_account_info(mint)?.is_initialized() {
        let pointer = pinocchio_token_2022::extension::scaled_ui_amount::state::ScaledUiAmountConfig::from_account_info(mint)?;

        if pointer.authority() != authority.as_ref() {
            Err(ProgramError::InvalidAccountData)?
        }

        if pointer.multiplier() != f64::from(multiplier) {
            Err(ProgramError::InvalidAccountData)?
        }

        return Ok(());
    }

    scaled_ui_amount::InitializeScaledUiAmount {
        mint_account: mint,
        authority,
        multiplier: multiplier.into(),
        token_program: &token_program.key(),
    }
    .invoke()
}
