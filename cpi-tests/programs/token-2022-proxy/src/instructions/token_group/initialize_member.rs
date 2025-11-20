use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022,
};

pub fn initialize_member(accounts: &[AccountInfo]) -> ProgramResult {
    let [member, member_mint, member_mint_authority, group, group_update_authority, token_program] =
        accounts
    else {
        Err(ProgramError::InvalidAccountData)?
    };

    if let Ok(token_group_member) =
        pinocchio_token_2022::extension::token_group::state::TokenGroupMember::from_account_info(
            member,
        )
    {
        if token_group_member.group() != group.key() {
            Err(ProgramError::InvalidAccountData)?
        }

        return Ok(());
    }

    pinocchio_token_2022::extension::token_group::InitializeMember {
        member,
        member_mint,
        member_mint_authority,
        group,
        group_update_authority,
        program_id: &token_program.key(),
    }
    .invoke()
}
