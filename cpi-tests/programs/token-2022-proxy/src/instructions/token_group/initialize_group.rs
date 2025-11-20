use {
    crate::helpers::from_optional_non_zero_pubkey,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022,
    spl_pod::{optional_keys::OptionalNonZeroPubkey, primitives::PodU64},
};

pub fn initialize_group(
    accounts: &[AccountInfo],
    update_authority: OptionalNonZeroPubkey,
    max_size: PodU64,
) -> ProgramResult {
    let [group, mint, mint_authority, token_program] = accounts else {
        Err(ProgramError::InvalidAccountData)?
    };

    let update_authority = from_optional_non_zero_pubkey(update_authority);
    let max_size: u64 = max_size.into();

    if let Ok(token_group) =
        pinocchio_token_2022::extension::token_group::state::TokenGroup::from_account_info(mint)
    {
        if token_group.update_authority() != update_authority.as_ref() {
            Err(ProgramError::InvalidAccountData)?
        }

        if token_group.max_size() != max_size {
            Err(ProgramError::InvalidAccountData)?
        }

        return Ok(());
    }

    pinocchio_token_2022::extension::token_group::InitializeGroup {
        group,
        mint,
        mint_authority,
        update_authority: update_authority.as_ref(),
        max_size,
        program_id: &token_program.key(),
    }
    .invoke()
}
