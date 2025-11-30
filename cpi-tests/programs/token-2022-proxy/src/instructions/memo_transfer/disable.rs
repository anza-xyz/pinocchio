use {
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    pinocchio_token_2022,
};

pub fn disable(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    // Expected account layout:
    //   [ token_account, authority/owner , ...signers, token_program ]
    //
    // Single authority case:
    //   [ token_account, authority/owner (signer), token_program ]
    //
    // Multisig case:
    //   [ token_account, authority/owner , signer1, signer2, ... signer n, token_program ]

    if accounts.len() < 3 {
        Err(ProgramError::NotEnoughAccountKeys)?;
    }

    let token_account = &accounts[0];
    let authority = &accounts[1];
    let token_program = accounts.last().unwrap(); // dev : since token_program is always the last account
    let signers = &accounts[2..accounts.len() - 1]; // dev : everything between authority and token_program ; note : in Single authority case: [2..2] will result in empty array

    match instruction_data[0] {
        // dev: use index 0 — lib.rs already strips the first byte (extension tag) from original instruction data; value `1` = Disable.
        1 => pinocchio_token_2022::extension::memo_transfer::Disable {
            token_account,
            authority,
            signers,
            token_program: token_program.key(),
        }
        .invoke(),

        _ => return Err(ProgramError::InvalidInstructionData),
    }
}
