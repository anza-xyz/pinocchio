use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022,
    spl_pod::primitives::PodU64,
};

pub fn update_max_size(accounts: &[AccountInfo], max_size: PodU64) -> ProgramResult {
    let [group, update_authority, token_program] = accounts else {
        Err(ProgramError::InvalidAccountData)?
    };

    let max_size: u64 = max_size.into();

    pinocchio_token_2022::extension::token_group::UpdateGroupMaxSize {
        group,
        update_authority,
        max_size,
        program_id: &token_program.key(),
    }
    .invoke()
}
