use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022,
    spl_token_2022_interface::instruction::decode_instruction_data,
};

pub fn update(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    // Expected account layout:
    //   [ mint, authority, ...signers, token_program ]
    //
    // Single authority case:
    //   [ mint, authority (signer), token_program ]
    //
    // Multisig case:
    //   [ mint, authority, signer1, signer2, ... signer n, token_program ]

    if accounts.len() < 3 {
        Err(ProgramError::NotEnoughAccountKeys)?;
    }

    let mint = &accounts[0];
    let authority = &accounts[1];
    let token_program = accounts.last().unwrap(); // dev : since token_program is always the last account
    let signers = &accounts[2..accounts.len() - 1]; // dev : everything between authority and token_program ; note : in Single authority case: [2..2] will result in empty array

    let &spl_token_2022_interface::extension::metadata_pointer::instruction::UpdateInstructionData {
        metadata_address,
    } = decode_instruction_data(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    pinocchio_token_2022::extension::metadata_pointer::Update {
        mint,
        authority,
        new_metadata_address: Option::<solana_address::Address>::from(metadata_address)
            .map(|x| x.to_bytes())
            .as_ref(),
        signers,
        token_program: token_program.key(),
    }
    .invoke()
}
