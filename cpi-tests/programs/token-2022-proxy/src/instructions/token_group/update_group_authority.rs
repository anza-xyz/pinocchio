use {
    crate::helpers::from_optional_non_zero_pubkey,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022,
    spl_pod::optional_keys::OptionalNonZeroPubkey,
};

pub fn update_group_authority(
    accounts: &[AccountInfo],
    new_authority: OptionalNonZeroPubkey,
) -> ProgramResult {
    let [group, current_authority, token_program] = accounts else {
        Err(ProgramError::InvalidAccountData)?
    };

    pinocchio_token_2022::extension::token_group::UpdateGroupAuthority {
        group,
        current_authority,
        new_authority: from_optional_non_zero_pubkey(new_authority).as_ref(),
        program_id: &token_program.key(),
    }
    .invoke()
}
