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

    let &spl_token_2022_interface::extension::metadata_pointer::instruction::InitializeInstructionData {
        authority,
        metadata_address,
    } = decode_instruction_data(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let authority = from_optional_non_zero_pubkey(authority);
    let metadata_address = from_optional_non_zero_pubkey(metadata_address);

    if pinocchio_token_2022::state::Mint::from_account_info(mint)?.is_initialized() {
        let pointer =
            pinocchio_token_2022::extension::metadata_pointer::state::MetadataPointer::from_account_info(mint)?;

        if pointer.authority() != authority.as_ref() {
            Err(ProgramError::InvalidAccountData)?
        }

        if pointer.metadata_address() != metadata_address.as_ref() {
            Err(ProgramError::InvalidAccountData)?
        }

        return Ok(());
    }

    pinocchio_token_2022::extension::metadata_pointer::Initialize {
        mint,
        authority: authority.as_ref(),
        metadata_address: metadata_address.as_ref(),
        token_program: &token_program.key(),
    }
    .invoke()
}
