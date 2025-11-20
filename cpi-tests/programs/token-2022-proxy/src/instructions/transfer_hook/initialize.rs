use {
    crate::helpers::from_optional_non_zero_pubkey,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022::extension::transfer_hook,
    spl_token_2022_interface::instruction::decode_instruction_data,
};

pub fn initialize(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let [mint, token_program] = accounts else {
        Err(ProgramError::InvalidAccountData)?
    };

    let &spl_token_2022_interface::extension::transfer_hook::instruction::InitializeInstructionData {
        authority,
        program_id,
    } = decode_instruction_data(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let authority = from_optional_non_zero_pubkey(authority);
    let program_id = from_optional_non_zero_pubkey(program_id);

    if pinocchio_token_2022::state::Mint::from_account_info(mint)?.is_initialized() {
        let transfer_hook_config = pinocchio_token_2022::extension::transfer_hook::state::TransferHook::from_account_info(mint)?;

        if transfer_hook_config.authority() != authority.as_ref() {
            Err(ProgramError::InvalidAccountData)?
        }

        return Ok(());
    }

    transfer_hook::InitializeTransferHook {
        mint_account: mint,
        authority: authority.as_ref(),
        program_id: program_id.as_ref(),
        token_program: &token_program.key(),
    }
    .invoke()
}

