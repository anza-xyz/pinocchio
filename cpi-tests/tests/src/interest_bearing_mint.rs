use {
    crate::helpers::{
        extensions::token_2022::{
            initialize_mint::Token2022InitializeMintExtension,
            initialize_multisig::Token2022InitializeMultisigExtension,
            interest_bearing_mint::Token2022InterestBearingMintExtension,
        },
        suite::{
            core::App,
            types::{pin_pubkey_to_addr, AppUser, PinPubkey, Target, TestResult},
        },
    },
    pretty_assertions::assert_eq,
    solana_signer::Signer,
    spl_pod::optional_keys::OptionalNonZeroPubkey,
    spl_token_2022_interface::extension::{
        interest_bearing_mint::InterestBearingConfig, ExtensionType,
    },
};

#[test]
fn initialize_interest_bearing_mint_with_default_authority() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::InterestBearingConfig]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let interest_bearing_config = InterestBearingConfig {
        rate_authority: OptionalNonZeroPubkey::default(),
        initialization_timestamp: 0.into(),
        last_update_timestamp: 0.into(),
        pre_update_average_rate: 500.into(),
        current_rate: 500.into(),
    };

    app.token_2022_try_initialize_interest_bearing_mint(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        None,
        500,
    )?;

    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    assert_eq!(
        app.token_2022_query_interest_bearing_mint(Target::Spl, mint_pubkey)?,
        interest_bearing_config
    );

    Ok(())
}

#[test]
fn proxy_initialize_interest_bearing_mint_with_default_authority() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::InterestBearingConfig]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let interest_bearing_config = InterestBearingConfig {
        rate_authority: OptionalNonZeroPubkey::default(),
        initialization_timestamp: 0.into(),
        last_update_timestamp: 0.into(),
        pre_update_average_rate: 500.into(),
        current_rate: 500.into(),
    };

    app.token_2022_try_initialize_interest_bearing_mint(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        None,
        500,
    )?;

    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    assert_eq!(
        app.token_2022_query_interest_bearing_mint(Target::Spl, mint_pubkey)?,
        interest_bearing_config
    );
    assert_eq!(
        app.token_2022_query_interest_bearing_mint(Target::Proxy, mint_pubkey)?,
        interest_bearing_config
    );

    Ok(())
}

#[test]
fn initialize_and_update_interest_bearing_mint() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::InterestBearingConfig]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let interest_bearing_config = InterestBearingConfig {
        rate_authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&mint_authority.pubkey())),
        initialization_timestamp: 0.into(),
        last_update_timestamp: 0.into(),
        pre_update_average_rate: 500.into(),
        current_rate: 500.into(),
    };

    app.token_2022_try_initialize_interest_bearing_mint(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        Some(&mint_authority.pubkey()),
        500,
    )?;

    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    assert_eq!(
        app.token_2022_query_interest_bearing_mint(Target::Spl, mint_pubkey)?,
        interest_bearing_config
    );

    app.token_2022_try_update_interest_bearing_mint_rate(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        &mint_authority.pubkey(),
        750,
    )?;

    assert_eq!(
        app.token_2022_query_interest_bearing_mint(Target::Spl, mint_pubkey)?,
        InterestBearingConfig {
            rate_authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&mint_authority.pubkey())),
            initialization_timestamp: 0.into(),
            last_update_timestamp: 0.into(),
            pre_update_average_rate: 500.into(),
            current_rate: 750.into(),
        }
    );

    Ok(())
}

#[test]
fn proxy_initialize_and_update_interest_bearing_mint() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::InterestBearingConfig]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let interest_bearing_config = InterestBearingConfig {
        rate_authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&mint_authority.pubkey())),
        initialization_timestamp: 0.into(),
        last_update_timestamp: 0.into(),
        pre_update_average_rate: 500.into(),
        current_rate: 500.into(),
    };

    app.token_2022_try_initialize_interest_bearing_mint(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        Some(&mint_authority.pubkey()),
        500,
    )?;

    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    // execute 2nd time to run internal checks
    app.token_2022_try_initialize_interest_bearing_mint(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        Some(&mint_authority.pubkey()),
        500,
    )?;

    assert_eq!(
        app.token_2022_query_interest_bearing_mint(Target::Spl, mint_pubkey)?,
        interest_bearing_config
    );
    assert_eq!(
        app.token_2022_query_interest_bearing_mint(Target::Proxy, mint_pubkey)?,
        interest_bearing_config
    );

    app.token_2022_try_update_interest_bearing_mint_rate(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        &mint_authority.pubkey(),
        750,
    )?;

    assert_eq!(
        app.token_2022_query_interest_bearing_mint(Target::Spl, mint_pubkey)?,
        InterestBearingConfig {
            rate_authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&mint_authority.pubkey())),
            initialization_timestamp: 0.into(),
            last_update_timestamp: 0.into(),
            pre_update_average_rate: 500.into(),
            current_rate: 750.into(),
        }
    );

    Ok(())
}

#[test]
fn proxy_initialize_and_update_interest_bearing_mint_with_multisig() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::InterestBearingConfig]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;

    // create a multisig authority with 3 signers, requiring 2 signatures
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
        &[signer1.pubkey(), signer2.pubkey(), signer3.pubkey()],
    )?;

    let multisig_authority = multisig_kp.pubkey().to_bytes().into();
    let freeze_authority = Some(AppUser::Admin.pubkey());
    let initial_rate = 500;

    // initialize interest bearing mint with multisig authority
    app.token_2022_try_initialize_interest_bearing_mint(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        Some(&multisig_authority),
        initial_rate,
    )?;

    app.token_2022_try_initialize_mint(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &multisig_authority,
        freeze_authority.as_ref(),
    )?;

    assert_eq!(
        app.token_2022_query_interest_bearing_mint(Target::Proxy, mint_pubkey)?,
        InterestBearingConfig {
            rate_authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&multisig_authority)),
            initialization_timestamp: 0.into(),
            last_update_timestamp: 0.into(),
            pre_update_average_rate: initial_rate.into(),
            current_rate: initial_rate.into(),
        }
    );

    // update interest rate with sufficient multisig signers (signer1 + signer2)
    let new_rate = 750;
    app.token_2022_try_update_interest_bearing_mint_rate_multisig(
        Target::Proxy,
        mint_pubkey,
        &multisig_authority,
        &[signer1, signer2],
        new_rate,
    )?;

    assert_eq!(
        app.token_2022_query_interest_bearing_mint(Target::Proxy, mint_pubkey)?,
        InterestBearingConfig {
            rate_authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&multisig_authority)),
            initialization_timestamp: 0.into(),
            last_update_timestamp: 0.into(),
            pre_update_average_rate: initial_rate.into(),
            current_rate: new_rate.into(),
        }
    );

    // update to different rate with different signers (signer2 + signer3)
    let final_rate = 1000;
    app.token_2022_try_update_interest_bearing_mint_rate_multisig(
        Target::Proxy,
        mint_pubkey,
        &multisig_authority,
        &[signer2, signer3],
        final_rate,
    )?;

    assert_eq!(
        app.token_2022_query_interest_bearing_mint(Target::Proxy, mint_pubkey)?,
        InterestBearingConfig {
            rate_authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&multisig_authority)),
            initialization_timestamp: 0.into(),
            last_update_timestamp: 0.into(),
            pre_update_average_rate: new_rate.into(),
            current_rate: final_rate.into(),
        }
    );

    Ok(())
}
