use {
    crate::helpers::from_c_option,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022,
    solana_address::Address,
    solana_program_option::COption,
};

pub fn initialize_mint(
    accounts: &[AccountInfo],
    decimals: u8,
    mint_authority: Address,
    freeze_authority: COption<Address>,
) -> ProgramResult {
    let [mint, rent_sysvar, token_program] = accounts else {
        Err(ProgramError::InvalidAccountData)?
    };

    // if account is initialized just check from_account_info and fields
    {
        let mint_state = pinocchio_token_2022::state::Mint::from_account_info(mint)?;
        if mint_state.is_initialized() {
            if mint_state.decimals() != decimals {
                Err(ProgramError::InvalidAccountData)?;
            }

            if mint_state.mint_authority() != Some(&mint_authority.to_bytes()) {
                Err(ProgramError::InvalidAccountData)?;
            }

            if mint_state.freeze_authority()
                != from_c_option(freeze_authority.map(|x| x.to_bytes()).as_ref())
            {
                Err(ProgramError::InvalidAccountData)?;
            }

            return Ok(());
        }
    }

    pinocchio_token_2022::instructions::InitializeMint {
        mint,
        rent_sysvar,
        decimals,
        mint_authority: &mint_authority.to_bytes(),
        freeze_authority: from_c_option(freeze_authority)
            .map(|x| x.to_bytes())
            .as_ref(),
        token_program: &token_program.key(),
    }
    .invoke()
}
