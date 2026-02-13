use mollusk_svm::{result::Check, Mollusk};
use solana_sdk::{
    account::{AccountSharedData, ReadableAccount},
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

const PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
    0x1f, 0x20,
]);

#[test]
fn test_update_value_cu() {
    let mollusk = Mollusk::new(&PROGRAM_ID, "target/deploy/pinocchio_small_oracle");

    let authority = Pubkey::new_unique();
    let state_pubkey = Pubkey::new_unique();

    let state_account = {
        let mut account = AccountSharedData::new(1_000_000_000, 40, &PROGRAM_ID);
        let mut data = Vec::with_capacity(40);
        data.extend_from_slice(&authority.to_bytes());
        data.extend_from_slice(&0u64.to_le_bytes());
        account.set_data_from_slice(&data);
        account
    };

    let new_value: u64 = 42;
    let instruction = Instruction::new_with_bytes(
        PROGRAM_ID,
        &new_value.to_le_bytes(),
        vec![
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(state_pubkey, false),
        ],
    );

    let result = mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (
                authority,
                AccountSharedData::new(1_000_000_000, 0, &Pubkey::default()),
            ),
            (state_pubkey, state_account),
        ],
        &[Check::success()],
    );

    println!("========================================");
    println!(
        "UpdateValue CU consumption: {}",
        result.compute_units_consumed
    );
    println!("========================================");

    assert!(
        result.compute_units_consumed < 200,
        "UpdateValue used {} CUs, expected < 200",
        result.compute_units_consumed
    );

    let result_data = result.get_account(&state_pubkey).unwrap().data();
    let result_value = u64::from_le_bytes(result_data[32..40].try_into().unwrap());
    assert_eq!(result_value, 42);
}
