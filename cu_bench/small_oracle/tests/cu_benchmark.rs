use std::path::PathBuf;

use mollusk_svm::{result::Check, Mollusk};
use solana_sdk::{
    account::{AccountSharedData, ReadableAccount},
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use pinocchio_small_oracle::{SMALL_ORACLE_ACCOUNT_SIZE, SMALL_ORACLE_VALUE_SIZE};

const PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
]);
const VALUE_TO_WRITE: u64 = 42;

fn deploy_path(deploy_binary: &str, caller_manifest_dir: &str) -> String {
    let mut caller_path = PathBuf::from(caller_manifest_dir);
    if !caller_path.is_absolute() {
        caller_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(&caller_path);
    }

    let caller_path = caller_path.canonicalize().unwrap_or_else(|_| caller_path);

    for ancestor in caller_path.ancestors() {
        let deploy_dir = ancestor.join("target").join("deploy");
        let bare_binary_path = deploy_dir.join(deploy_binary);
        if bare_binary_path.exists() {
            return bare_binary_path.to_string_lossy().into_owned();
        }

        let so_binary_path = deploy_dir.join(format!("{deploy_binary}.so"));
        if so_binary_path.exists() {
            return bare_binary_path.to_string_lossy().into_owned();
        }
    }

    panic!(
        "Could not find {deploy_binary}. Build with `make sbf` and rerun tests."
    );
}

fn run_cu_benchmark(variant_name: &str, caller_manifest_dir: &str, deploy_binary: &str) {
    let mollusk = Mollusk::new(&PROGRAM_ID, &deploy_path(deploy_binary, caller_manifest_dir));

    let authority = Pubkey::new_unique();
    let state_pubkey = Pubkey::new_unique();

    let state_account = {
        let mut account = AccountSharedData::new(1_000_000_000, SMALL_ORACLE_ACCOUNT_SIZE, &PROGRAM_ID);
        let mut data = Vec::with_capacity(SMALL_ORACLE_ACCOUNT_SIZE);
        data.extend_from_slice(&authority.to_bytes());
        data.extend_from_slice(&0u64.to_le_bytes());
        account.set_data_from_slice(&data);
        account
    };

    let value = VALUE_TO_WRITE.to_le_bytes();
    let instruction = Instruction::new_with_bytes(
        PROGRAM_ID,
        &value,
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
        "{} - UpdateValue CU consumption: {}",
        variant_name, result.compute_units_consumed
    );
    println!("========================================");

    assert!(
        result.compute_units_consumed < 200,
        "UpdateValue used {} CUs, expected < 200",
        result.compute_units_consumed
    );

    let result_data = result.get_account(&state_pubkey).unwrap().data();
    let value_offset = SMALL_ORACLE_ACCOUNT_SIZE - SMALL_ORACLE_VALUE_SIZE;
    let result_value = u64::from_le_bytes(
        result_data[value_offset..SMALL_ORACLE_ACCOUNT_SIZE]
            .try_into()
            .unwrap(),
    );
    assert_eq!(result_value, VALUE_TO_WRITE);
}

#[cfg(feature = "opt")]
#[test]
fn test_update_value_cu() {
    run_cu_benchmark("OPT", env!("CARGO_MANIFEST_DIR"), "pinocchio_small_oracle");
}

#[cfg(feature = "naive")]
#[test]
fn test_update_value_cu() {
    run_cu_benchmark("NAIVE", env!("CARGO_MANIFEST_DIR"), "pinocchio_small_oracle");
}

#[cfg(feature = "manual")]
#[test]
fn test_update_value_cu() {
    run_cu_benchmark("MANUAL", env!("CARGO_MANIFEST_DIR"), "pinocchio_small_oracle");
}
