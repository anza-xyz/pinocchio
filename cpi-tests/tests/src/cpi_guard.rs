use {
    crate::helpers::{
        extensions::token_2022::{
            cpi_guard::Token2022CpiGuardExtension,
            initialize_mint::Token2022InitializeMintExtension,
            initialize_multisig::Token2022InitializeMultisigExtension,
            token_account::Token2022TokenAccountExtension,
        },
        suite::{
            core::App,
            types::{AppUser, SolPubkey, Target, TestError, TestResult},
        },
    },
    pretty_assertions::assert_eq,
    solana_signer::Signer,
    spl_token_2022_interface::extension::ExtensionType,
};

#[test]
fn cpi_guard_enable_and_disable() -> TestResult<()> {
    let mut app = App::new(false);
    let owner = AppUser::Admin;

    // Create mint account without extensions
    let (_, mint_kp) = app.token_2022_try_create_mint_account(owner, None, None)?;
    let mint_pubkey = &mint_kp.pubkey().to_bytes();

    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey().to_bytes());

    // Initialize the mint
    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        &mint_pubkey,
        decimals,
        &mint_authority.pubkey().to_bytes(),
        freeze_authority.as_ref(),
    )?;

    // Create token account with CpiGuard extension space
    let (_, token_account_kp) = app.token_2022_try_create_and_init_token_account(
        AppUser::Admin,
        &owner.pubkey().to_bytes(),
        &mint_pubkey,
        &[ExtensionType::CpiGuard],
    )?;
    let token_account_pubkey = &token_account_kp.pubkey().to_bytes();

    // Enable CPI Guard
    app.token_2022_try_enable_cpi_guard(Target::Spl, AppUser::Admin, token_account_pubkey)?;

    // Verify CPI Guard is enabled
    assert_eq!(
        app.token_2022_query_cpi_guard(Target::Spl, token_account_pubkey)
            .map(|x| Into::<bool>::into(x.lock_cpi))?,
        true
    );

    // Disable CPI Guard
    app.token_2022_try_disable_cpi_guard(Target::Spl, AppUser::Admin, token_account_pubkey)?;

    // Verify CPI Guard is disabled
    assert_eq!(
        app.token_2022_query_cpi_guard(Target::Spl, token_account_pubkey)
            .map(|x| Into::<bool>::into(x.lock_cpi))?,
        false
    );

    Ok(())
}

#[test]
fn proxy_cpi_guard_enable_and_disable() -> TestResult<()> {
    let mut app = App::new(false);
    let owner = AppUser::Admin;

    // Create mint account without extensions
    let (_, mint_kp) = app.token_2022_try_create_mint_account(owner, None, None)?;
    let mint_pubkey = &mint_kp.pubkey().to_bytes();

    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey().to_bytes());

    // Initialize the mint
    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        &mint_pubkey,
        decimals,
        &mint_authority.pubkey().to_bytes(),
        freeze_authority.as_ref(),
    )?;

    // Create token account with CpiGuard extension space
    let (_, token_account_kp) = app.token_2022_try_create_and_init_token_account(
        AppUser::Admin,
        &owner.pubkey().to_bytes(),
        &mint_pubkey,
        &[ExtensionType::CpiGuard],
    )?;
    let token_account_pubkey = &token_account_kp.pubkey().to_bytes();

    // Try enable CPI Guard - CPI Guard status cannot be changed in CPI
    let res = app
        .token_2022_try_enable_cpi_guard(Target::Proxy, AppUser::Admin, token_account_pubkey)
        .unwrap_err();
    assert_eq!(
        res,
        TestError {
            info: "custom program error: 0x29".to_string(),
            index: None,
        },
    );

    // Try disable CPI Guard - CPI Guard status cannot be changed in CPI
    let res = app
        .token_2022_try_disable_cpi_guard(Target::Proxy, AppUser::Admin, token_account_pubkey)
        .unwrap_err();
    assert_eq!(
        res,
        TestError {
            info: "custom program error: 0x29".to_string(),
            index: None,
        },
    );

    // Verify CPI Guard is disabled
    assert_eq!(
        app.token_2022_query_cpi_guard(Target::Proxy, token_account_pubkey)
            .map(|x| Into::<bool>::into(x.lock_cpi))?,
        false
    );

    // Enable CPI Guard
    app.token_2022_try_enable_cpi_guard(Target::Spl, AppUser::Admin, token_account_pubkey)?;

    // Verify CPI Guard is enabled
    assert_eq!(
        app.token_2022_query_cpi_guard(Target::Proxy, token_account_pubkey)
            .map(|x| Into::<bool>::into(x.lock_cpi))?,
        true
    );

    // execute 2nd time to run internal checks
    app.token_2022_try_enable_cpi_guard(Target::Proxy, AppUser::Admin, token_account_pubkey)?;

    Ok(())
}

#[test]
fn cpi_guard_enable_and_disable_multisig() -> TestResult<()> {
    let mut app = App::new(false);

    // Create a multisig authority with 3 signers, requiring 2 signatures
    let signer1 = AppUser::Admin;
    let signer2 = AppUser::Alice;
    let signer3 = AppUser::Bob;
    let required_signers: u8 = 2;

    let (_, multisig_kp) = app.token_2022_try_create_multisig(AppUser::Admin, None)?;
    let multisig_pubkey = &multisig_kp.pubkey().to_bytes().into();
    app.token_2022_try_initialize_multisig(
        Target::Spl,
        AppUser::Admin,
        multisig_pubkey,
        required_signers,
        &[
            signer1.pubkey().to_bytes(),
            signer2.pubkey().to_bytes(),
            signer3.pubkey().to_bytes(),
        ],
    )?;

    let multisig_authority = &multisig_kp.pubkey().to_bytes().into();

    // Create mint account without extensions
    let (_, mint_kp) = app.token_2022_try_create_mint_account(AppUser::Admin, None, None)?;
    let mint_pubkey = &mint_kp.pubkey().to_bytes();

    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey().to_bytes());

    // Initialize the mint
    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        &mint_pubkey,
        decimals,
        &mint_authority.pubkey().to_bytes(),
        freeze_authority.as_ref(),
    )?;

    // Create token account with CpiGuard extension space
    let (_, token_account_kp) = app.token_2022_try_create_and_init_token_account(
        AppUser::Admin,
        &multisig_authority,
        &mint_pubkey,
        &[ExtensionType::CpiGuard],
    )?;
    let token_account_pubkey = &token_account_kp.pubkey().to_bytes();

    // Enable CPI Guard
    app.token_2022_try_enable_cpi_guard_mutisig(
        Target::Spl,
        token_account_pubkey,
        multisig_authority,
        &[signer1, signer2],
    )?;

    // Verify CPI Guard is enabled
    assert_eq!(
        app.token_2022_query_cpi_guard(Target::Spl, token_account_pubkey)
            .map(|x| Into::<bool>::into(x.lock_cpi))?,
        true
    );

    // Disable CPI Guard
    app.token_2022_try_disable_cpi_guard_mutisig(
        Target::Spl,
        token_account_pubkey,
        multisig_authority,
        &[signer1, signer2],
    )?;

    // Verify CPI Guard is disabled
    assert_eq!(
        app.token_2022_query_cpi_guard(Target::Spl, token_account_pubkey)
            .map(|x| Into::<bool>::into(x.lock_cpi))?,
        false
    );

    Ok(())
}

#[test]
fn proxy_cpi_guard_enable_and_disable_multisig() -> TestResult<()> {
    let mut app = App::new(false);

    // Create a multisig authority with 3 signers, requiring 2 signatures
    let signer1 = AppUser::Admin;
    let signer2 = AppUser::Alice;
    let signer3 = AppUser::Bob;
    let required_signers: u8 = 2;

    let (_, multisig_kp) = app.token_2022_try_create_multisig(AppUser::Admin, None)?;
    let multisig_pubkey = &multisig_kp.pubkey().to_bytes().into();
    app.token_2022_try_initialize_multisig(
        Target::Spl,
        AppUser::Admin,
        multisig_pubkey,
        required_signers,
        &[
            signer1.pubkey().to_bytes(),
            signer2.pubkey().to_bytes(),
            signer3.pubkey().to_bytes(),
        ],
    )?;

    let multisig_authority = &multisig_kp.pubkey().to_bytes().into();

    // Create mint account without extensions
    let (_, mint_kp) = app.token_2022_try_create_mint_account(AppUser::Admin, None, None)?;
    let mint_pubkey = &mint_kp.pubkey().to_bytes();

    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey().to_bytes());

    // Initialize the mint
    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        &mint_pubkey,
        decimals,
        &mint_authority.pubkey().to_bytes(),
        freeze_authority.as_ref(),
    )?;

    // Create token account with CpiGuard extension space
    let (_, token_account_kp) = app.token_2022_try_create_and_init_token_account(
        AppUser::Admin,
        &multisig_authority,
        &mint_pubkey,
        &[ExtensionType::CpiGuard],
    )?;
    let token_account_pubkey = &token_account_kp.pubkey().to_bytes();

    // Try enable CPI Guard - CPI Guard status cannot be changed in CPI
    let res = app
        .token_2022_try_enable_cpi_guard_mutisig(
            Target::Proxy,
            token_account_pubkey,
            multisig_authority,
            &[signer1, signer2],
        )
        .unwrap_err();
    assert_eq!(
        res,
        TestError {
            info: "custom program error: 0x29".to_string(),
            index: None,
        },
    );

    Ok(())
}
