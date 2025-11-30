use {
    crate::helpers::{
        extensions::token_2022::{
            scaled_ui_amount::Token2022ScaledUiAmountExtension,
            initialize_mint::Token2022InitializeMintExtension,
            initialize_multisig::Token2022InitializeMultisigExtension,
        },
        suite::{
            core::App,
            types::{AppUser, PinPubkey, Target, TestResult},
        },
    },
    pretty_assertions::assert_eq,
    solana_signer::Signer,
    spl_token_2022_interface::extension::{scaled_ui_amount::ScaledUiAmountConfig, ExtensionType},
};

#[test]
fn initialize_scaled_ui_amount_with_default_authority() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::ScaledUiAmount]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let scaled_ui_amount_config = ScaledUiAmountConfig {
        authority: spl_pod::optional_keys::OptionalNonZeroPubkey::try_from(Some(solana_address::Address::new_from_array(mint_authority.pubkey()))).unwrap(),
        multiplier: 1.0.into(),
        new_multiplier_effective_timestamp: 0.into(),
        new_multiplier: 1.0.into(),
    };

    app.token_2022_try_initialize_scaled_ui_amount(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        &mint_authority.pubkey(),
        1.0,
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
        app.token_2022_query_scaled_ui_amount(Target::Spl, mint_pubkey)?,
        scaled_ui_amount_config
    );

    Ok(())
}

#[test]
fn proxy_initialize_scaled_ui_amount_with_default_authority() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::ScaledUiAmount]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let scaled_ui_amount_config = ScaledUiAmountConfig {
        authority: spl_pod::optional_keys::OptionalNonZeroPubkey::try_from(Some(solana_address::Address::new_from_array(mint_authority.pubkey()))).unwrap(),
        multiplier: 1.0.into(),
        new_multiplier_effective_timestamp: 0.into(),
        new_multiplier: 1.0.into(),
    };

    app.token_2022_try_initialize_scaled_ui_amount(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        &mint_authority.pubkey(),
        1.0,
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
        app.token_2022_query_scaled_ui_amount(Target::Proxy, mint_pubkey)?,
        scaled_ui_amount_config
    );

    assert_eq!(
        app.token_2022_query_scaled_ui_amount(Target::Spl, mint_pubkey)?,
        scaled_ui_amount_config
    );

    Ok(())
}

#[test]
fn initialize_and_update_scaled_ui_amount() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::ScaledUiAmount]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    // Initialize with multiplier 1.0
    app.token_2022_try_initialize_scaled_ui_amount(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        &mint_authority.pubkey(),
        1.0,
    )?;

    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    // Verify initial state
    let initial_config = ScaledUiAmountConfig {
        authority: spl_pod::optional_keys::OptionalNonZeroPubkey::try_from(Some(solana_address::Address::new_from_array(mint_authority.pubkey()))).unwrap(),
        multiplier: 1.0.into(),
        new_multiplier_effective_timestamp: 0.into(),
        new_multiplier: 1.0.into(),
    };

    assert_eq!(
        app.token_2022_query_scaled_ui_amount(Target::Spl, mint_pubkey)?,
        initial_config
    );

    // Update multiplier to 2.5 with effective timestamp 1000
    app.token_2022_try_update_multiplier(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        &mint_authority.pubkey(),
        2.5,
        1000,
    )?;

    // Verify updated state
    let updated_config = ScaledUiAmountConfig {
        authority: spl_pod::optional_keys::OptionalNonZeroPubkey::try_from(Some(solana_address::Address::new_from_array(mint_authority.pubkey()))).unwrap(),
        multiplier: 1.0.into(),
        new_multiplier_effective_timestamp: 1000.into(),
        new_multiplier: 2.5.into(),
    };

    assert_eq!(
        app.token_2022_query_scaled_ui_amount(Target::Spl, mint_pubkey)?,
        updated_config
    );

    Ok(())
}

#[test]
fn initialize_and_update_scaled_ui_amount_multisig() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::ScaledUiAmount]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

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
    let signers = &[signer1, signer2];

    // Initialize with multiplier 1.0 using multisig authority
    app.token_2022_try_initialize_scaled_ui_amount(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        &multisig_authority,
        1.0,
    )?;

    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    // Verify initial state
    let initial_config = ScaledUiAmountConfig {
        authority: spl_pod::optional_keys::OptionalNonZeroPubkey::try_from(Some(solana_address::Address::new_from_array(multisig_authority))).unwrap(),
        multiplier: 1.0.into(),
        new_multiplier_effective_timestamp: 0.into(),
        new_multiplier: 1.0.into(),
    };

    assert_eq!(
        app.token_2022_query_scaled_ui_amount(Target::Spl, mint_pubkey)?,
        initial_config
    );

    // Update multiplier using multisig to 3.0 with effective timestamp 2000
    app.token_2022_try_update_multiplier_multisig(
        Target::Spl,
        mint_pubkey,
        &multisig_authority,
        signers,
        3.0,
        2000,
    )?;

    // Verify updated state
    let updated_config = ScaledUiAmountConfig {
        authority: spl_pod::optional_keys::OptionalNonZeroPubkey::try_from(Some(solana_address::Address::new_from_array(multisig_authority))).unwrap(),
        multiplier: 1.0.into(),
        new_multiplier_effective_timestamp: 2000.into(),
        new_multiplier: 3.0.into(),
    };

    assert_eq!(
        app.token_2022_query_scaled_ui_amount(Target::Spl, mint_pubkey)?,
        updated_config
    );

    Ok(())
}
