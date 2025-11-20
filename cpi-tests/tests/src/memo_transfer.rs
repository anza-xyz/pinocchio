use {
    crate::helpers::{
        extensions::token_2022::{
            initialize_multisig::Token2022InitializeMultisigExtension,
            initialize_token_account::Token2022InitializeAccountExtension,
            memo_transfer::{MemoStatus, Token2022MemoTransferExtension},
        },
        suite::{
            core::{extension::get_account_data, App},
            types::{AppUser, PinPubkey, Target, TestResult},
        },
    },
    pretty_assertions::assert_eq,
    solana_signer::Signer,
    spl_token_2022_interface::extension::ExtensionType,
};

//// chore: maybe add tests to check transfer behavior across different MemoStatuses;
//// like make sure transfer fails when memo is enabled but the instruction doesn’t include one.
//// not really necessary though — fine to skip,  since memo extension data validation (data[165..171]) is enough.
#[test]
fn enable_memo_transfer_with_eoa() -> TestResult<()> {
    let mut app = App::new(false);

    let (_, mint) = app.token2022_try_create_and_try_initialize_mint(Target::Spl)?;

    let (_, token_account_keypair) = app.token_2022_try_create_token_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::MemoTransfer]),
    )?;
    let token_account = token_account_keypair.pubkey().to_bytes();
    let alice = AppUser::Alice.pubkey();

    app.token_2022_try_initialize_token_account(
        Target::Spl,
        AppUser::Admin,
        &token_account,
        &mint,
        &alice,
    )?;

    let mut token_account_raw_data = get_account_data(&app, &token_account)?;

    // assertion
    assert_eq!(
        MemoStatus::check_memo_status(&token_account_raw_data[165..171]),
        MemoStatus::Initialized
    );

    // Enable Memo
    app.token_2022_try_enable_memo_transfer(Target::Spl, &token_account, &alice, AppUser::Alice)?;
    token_account_raw_data = get_account_data(&app, &token_account)?;

    // assertion
    assert_eq!(
        MemoStatus::check_memo_status(&token_account_raw_data[165..171]),
        MemoStatus::Enabled
    );

    Ok(())
}

#[test]
fn enable_memo_transfer_with_multisig() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint) = app.token2022_try_create_and_try_initialize_mint(Target::Spl)?;

    // create a multisig authority with 3 signers, requiring 2 signatures
    let signer1 = AppUser::Admin;
    let signer2 = AppUser::Alice;
    let signer3 = AppUser::Bob;
    let required_signers: u8 = 2;

    let (_, multisig_kp) = app.token_2022_try_create_multisig(AppUser::Admin, None)?;
    let multisig_pubkey: &[u8; 32] = &multisig_kp.pubkey().to_bytes().into();
    app.token_2022_try_initialize_multisig(
        Target::Spl,
        AppUser::Admin,
        multisig_pubkey,
        required_signers,
        &[signer1.pubkey(), signer2.pubkey(), signer3.pubkey()],
    )?;

    let (_, token_account_keypair) = app.token_2022_try_create_token_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::MemoTransfer]),
    )?;
    let token_account = token_account_keypair.pubkey().to_bytes();

    app.token_2022_try_initialize_token_account(
        Target::Spl,
        AppUser::Admin,
        &token_account,
        &mint,
        &multisig_pubkey,
    )?;

    app.token_2022_try_enable_memo_transfer_multisig(
        Target::Spl,
        &token_account,
        &multisig_pubkey,
        &[signer1, signer2],
    )?;

    let token_account_raw_data = get_account_data(&app, &token_account)?;
    let memo_data_slice = &token_account_raw_data[165..171];

    assert_eq!(
        MemoStatus::check_memo_status(memo_data_slice),
        MemoStatus::Enabled
    );

    Ok(())
}

#[test]
fn disable_memo_transfer_with_eoa() -> TestResult<()> {
    let mut app = App::new(false);

    let (_, mint) = app.token2022_try_create_and_try_initialize_mint(Target::Spl)?;

    let (_, token_account_keypair) = app.token_2022_try_create_token_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::MemoTransfer]),
    )?;
    let token_account = token_account_keypair.pubkey().to_bytes();
    let alice = AppUser::Alice.pubkey();

    app.token_2022_try_initialize_token_account(
        Target::Spl,
        AppUser::Admin,
        &token_account,
        &mint,
        &alice,
    )?;

    app.token_2022_try_disable_memo_transfer(Target::Spl, &token_account, &alice, AppUser::Alice)?;

    let token_account_raw_data = get_account_data(&app, &token_account)?;
    let memo_data_slice = &token_account_raw_data[165..171];

    // assertion
    assert_eq!(
        MemoStatus::check_memo_status(memo_data_slice),
        MemoStatus::Disabled
    );

    Ok(())
}

#[test]
fn disable_memo_transfer_with_multisig() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint) = app.token2022_try_create_and_try_initialize_mint(Target::Spl)?;

    // create a multisig authority with 3 signers, requiring 2 signatures
    let signer1 = AppUser::Admin;
    let signer2 = AppUser::Alice;
    let signer3 = AppUser::Bob;
    let required_signers: u8 = 2;

    let (_, multisig_kp) = app.token_2022_try_create_multisig(AppUser::Admin, None)?;
    let multisig_pubkey: &[u8; 32] = &multisig_kp.pubkey().to_bytes().into();
    app.token_2022_try_initialize_multisig(
        Target::Spl,
        AppUser::Admin,
        multisig_pubkey,
        required_signers,
        &[signer1.pubkey(), signer2.pubkey(), signer3.pubkey()],
    )?;

    let (_, token_account_keypair) = app.token_2022_try_create_token_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::MemoTransfer]),
    )?;
    let token_account = token_account_keypair.pubkey().to_bytes();

    app.token_2022_try_initialize_token_account(
        Target::Spl,
        AppUser::Admin,
        &token_account,
        &mint,
        &multisig_pubkey,
    )?;

    app.token_2022_try_disable_memo_transfer_multisig(
        Target::Spl,
        &token_account,
        &multisig_pubkey,
        &[signer1, signer2],
    )?;

    let token_account_raw_data = get_account_data(&app, &token_account)?;
    let memo_data_slice = &token_account_raw_data[165..171];

    // assertion
    assert_eq!(
        MemoStatus::check_memo_status(memo_data_slice),
        MemoStatus::Disabled
    );

    Ok(())
}

#[test]
fn proxy_enable_memo_transfer_with_eoa() -> TestResult<()> {
    let mut app = App::new(false);

    let (_, mint) = app.token2022_try_create_and_try_initialize_mint(Target::Proxy)?;

    let (_, token_account_keypair) = app.token_2022_try_create_token_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::MemoTransfer]),
    )?;
    let token_account = token_account_keypair.pubkey().to_bytes();
    let alice = AppUser::Alice.pubkey();

    app.token_2022_try_initialize_token_account(
        Target::Proxy,
        AppUser::Admin,
        &token_account,
        &mint,
        &alice,
    )?;

    let mut token_account_raw_data = get_account_data(&app, &token_account)?;

    // assertion
    assert_eq!(
        MemoStatus::check_memo_status(&token_account_raw_data[165..171]),
        MemoStatus::Initialized
    );

    // Enable Memo
    app.token_2022_try_enable_memo_transfer(Target::Proxy, &token_account, &alice, AppUser::Alice)?;
    token_account_raw_data = get_account_data(&app, &token_account)?;

    // assertion
    assert_eq!(
        MemoStatus::check_memo_status(&token_account_raw_data[165..171]),
        MemoStatus::Enabled
    );

    Ok(())
}

#[test]
fn proxy_enable_memo_transfer_with_multisig() -> TestResult<()> {
    let mut app = App::new(true);
    let (_, mint) = app.token2022_try_create_and_try_initialize_mint(Target::Proxy)?;

    // create a multisig authority with 3 signers, requiring 2 signatures
    let signer1 = AppUser::Admin;
    let signer2 = AppUser::Alice;
    let signer3 = AppUser::Bob;
    let required_signers: u8 = 2;

    let (_, multisig_kp) = app.token_2022_try_create_multisig(AppUser::Admin, None)?;
    let multisig_pubkey: &[u8; 32] = &multisig_kp.pubkey().to_bytes().into();

    app.token_2022_try_initialize_multisig(
        Target::Spl, // dev: Using Target::Spl is fine here; routing through Proxy would need a dedicated InitializeMultisig helper.
        AppUser::Admin,
        multisig_pubkey,
        required_signers,
        &[signer1.pubkey(), signer2.pubkey(), signer3.pubkey()],
    )?;

    let (_, token_account_keypair) = app.token_2022_try_create_token_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::MemoTransfer]),
    )?;
    let token_account = token_account_keypair.pubkey().to_bytes();

    app.token_2022_try_initialize_token_account(
        Target::Proxy,
        AppUser::Admin,
        &token_account,
        &mint,
        &multisig_pubkey,
    )?;

    app.token_2022_try_enable_memo_transfer_multisig(
        Target::Proxy,
        &token_account,
        &multisig_pubkey,
        &[signer1, signer2],
    )?;

    let token_account_raw_data = get_account_data(&app, &token_account)?;
    let memo_data_slice = &token_account_raw_data[165..171];

    assert_eq!(
        MemoStatus::check_memo_status(memo_data_slice),
        MemoStatus::Enabled
    );

    Ok(())
}

#[test]
fn proxy_disable_memo_transfer_with_eoa() -> TestResult<()> {
    let mut app = App::new(false);

    let (_, mint) = app.token2022_try_create_and_try_initialize_mint(Target::Proxy)?;

    let (_, token_account_keypair) = app.token_2022_try_create_token_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::MemoTransfer]),
    )?;
    let token_account = token_account_keypair.pubkey().to_bytes();
    let alice = AppUser::Alice.pubkey();

    app.token_2022_try_initialize_token_account(
        Target::Proxy,
        AppUser::Admin,
        &token_account,
        &mint,
        &alice,
    )?;

    app.token_2022_try_disable_memo_transfer(
        Target::Proxy,
        &token_account,
        &alice,
        AppUser::Alice,
    )?;

    let token_account_raw_data = get_account_data(&app, &token_account)?;
    let memo_data_slice = &token_account_raw_data[165..171];

    // assertion
    assert_eq!(
        MemoStatus::check_memo_status(memo_data_slice),
        MemoStatus::Disabled
    );

    Ok(())
}

#[test]
fn proxy_disable_memo_transfer_with_multisig() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint) = app.token2022_try_create_and_try_initialize_mint(Target::Proxy)?;

    // create a multisig authority with 3 signers, requiring 2 signatures
    let signer1 = AppUser::Admin;
    let signer2 = AppUser::Alice;
    let signer3 = AppUser::Bob;
    let required_signers: u8 = 2;

    let (_, multisig_kp) = app.token_2022_try_create_multisig(AppUser::Admin, None)?;
    let multisig_pubkey: &[u8; 32] = &multisig_kp.pubkey().to_bytes().into();
    
    app.token_2022_try_initialize_multisig(
        Target::Spl, // dev: Using Target::Spl is fine here; routing through Proxy would need a dedicated InitializeMultisig helper.
        AppUser::Admin,
        multisig_pubkey,
        required_signers,
        &[signer1.pubkey(), signer2.pubkey(), signer3.pubkey()],
    )?;

    let (_, token_account_keypair) = app.token_2022_try_create_token_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::MemoTransfer]),
    )?;
    let token_account = token_account_keypair.pubkey().to_bytes();

    app.token_2022_try_initialize_token_account(
        Target::Proxy,
        AppUser::Admin,
        &token_account,
        &mint,
        &multisig_pubkey,
    )?;

    app.token_2022_try_disable_memo_transfer_multisig(
        Target::Proxy,
        &token_account,
        &multisig_pubkey,
        &[signer1, signer2],
    )?;

    let token_account_raw_data = get_account_data(&app, &token_account)?;
    let memo_data_slice = &token_account_raw_data[165..171];

    // assertion
    assert_eq!(
        MemoStatus::check_memo_status(memo_data_slice),
        MemoStatus::Disabled
    );

    Ok(())
}
