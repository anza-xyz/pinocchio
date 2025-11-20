use {
    crate::helpers::{
        extensions::token_2022::{
            pausable::Token2022PausableExtension,
            initialize_mint::Token2022InitializeMintExtension,
            initialize_multisig::Token2022InitializeMultisigExtension,
        },
        suite::{
            core::App,
            types::{pin_pubkey_to_addr, AppUser, PinPubkey, Target, TestResult},
        },
    },
    pretty_assertions::assert_eq,
    solana_signer::Signer,
    spl_token_2022_interface::extension::{pausable::PausableConfig, ExtensionType},
};

#[test]
fn initialize_pausable_with_authority() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::Pausable]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let pause_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    // Initialize pausable extension
    app.token_2022_try_initialize_pausable(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        &pause_authority.pubkey(),
    )?;

    // Initialize the mint
    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    // Query pausable config
    let pausable_config = app.token_2022_query_pausable_config(Target::Spl, mint_pubkey)?;
    
    assert_eq!(pausable_config.authority, spl_pod::optional_keys::OptionalNonZeroPubkey::try_from(Some(solana_address::Address::new_from_array(pause_authority.pubkey()))).unwrap());
    assert_eq!(pausable_config.paused, false.into());

    Ok(())
}

#[test]
fn initialize_pausable_proxy() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::Pausable]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let pause_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let pausable_config = PausableConfig {
        authority: spl_pod::optional_keys::OptionalNonZeroPubkey::try_from(Some(solana_address::Address::new_from_array(pause_authority.pubkey()))).unwrap(),
        paused: false.into(),
    };

    // Initialize pausable extension via proxy
    app.token_2022_try_initialize_pausable(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        &pause_authority.pubkey(),
    )?;

    // Initialize the mint
    app.token_2022_try_initialize_mint(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    // Query pausable config via proxy
    assert_eq!(
        app.token_2022_query_pausable_config(Target::Proxy, mint_pubkey)?,
        pausable_config
    );

    // verify with SPL query
    assert_eq!(
        app.token_2022_query_pausable_config(Target::Spl, mint_pubkey)?,
        pausable_config
    );

    Ok(())
}

#[test]
fn pause_and_resume_mint() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::Pausable]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let pause_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    // Initialize pausable extension
    app.token_2022_try_initialize_pausable(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        &pause_authority.pubkey(),
    )?;

    // Initialize the mint
    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    // Initially not paused
    let pausable_config = app.token_2022_query_pausable_config(Target::Spl, mint_pubkey)?;
    assert_eq!(pausable_config.paused, false.into());

    // Pause the mint
    app.token_2022_try_pause(
        Target::Spl,
        pause_authority,
        mint_pubkey,
        &pause_authority.pubkey(),
    )?;

    // Check that mint is paused
    let pausable_config = app.token_2022_query_pausable_config(Target::Spl, mint_pubkey)?;
    assert_eq!(pausable_config.paused, true.into());

    // Resume the mint
    app.token_2022_try_resume(
        Target::Spl,
        pause_authority,
        mint_pubkey,
        &pause_authority.pubkey(),
    )?;

    // Check that mint is resumed
    let pausable_config = app.token_2022_query_pausable_config(Target::Spl, mint_pubkey)?;
    assert_eq!(pausable_config.paused, false.into());

    Ok(())
}

#[test]
fn pause_and_resume_proxy() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::Pausable]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let pause_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    // Initialize pausable extension via proxy
    app.token_2022_try_initialize_pausable(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        &pause_authority.pubkey(),
    )?;

    // Initialize the mint via proxy
    app.token_2022_try_initialize_mint(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    // Initially not paused
    let pausable_config = app.token_2022_query_pausable_config(Target::Proxy, mint_pubkey)?;
    assert_eq!(pausable_config.paused, false.into());

    // Pause the mint via proxy
    app.token_2022_try_pause(
        Target::Proxy,
        pause_authority,
        mint_pubkey,
        &pause_authority.pubkey(),
    )?;

    // Check that mint is paused
    let pausable_config = app.token_2022_query_pausable_config(Target::Proxy, mint_pubkey)?;
    assert_eq!(pausable_config.paused, true.into());

    // Resume the mint via proxy
    app.token_2022_try_resume(
        Target::Proxy,
        pause_authority,
        mint_pubkey,
        &pause_authority.pubkey(),
    )?;

    // Check that mint is resumed
    let pausable_config = app.token_2022_query_pausable_config(Target::Proxy, mint_pubkey)?;
    assert_eq!(pausable_config.paused, false.into());

    Ok(())
}

#[test]
fn pause_and_resume_multisig() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::Pausable]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let pause_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    // Create multisig
    let (_, multisig_keypair) = app.token_2022_try_create_multisig(AppUser::Admin, None)?;
    let multisig_pubkey = &multisig_keypair.pubkey().to_bytes().into();
    app.token_2022_try_initialize_multisig(
        Target::Spl,
        AppUser::Admin,
        multisig_pubkey,
        1,
        &[AppUser::Admin.pubkey(), AppUser::Alice.pubkey()],
    )?;

    // Initialize pausable extension with multisig authority
    app.token_2022_try_initialize_pausable(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        &multisig_pubkey,
    )?;

    // Initialize the mint
    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    // Initially not paused
    let pausable_config = app.token_2022_query_pausable_config(Target::Spl, mint_pubkey)?;
    assert_eq!(pausable_config.paused, false.into());

    // Pause the mint with multisig
    app.token_2022_try_pause_multisig(
        Target::Spl,
        mint_pubkey,
        &multisig_pubkey,
        &[AppUser::Admin],
    )?;

    // Check that mint is paused
    let pausable_config = app.token_2022_query_pausable_config(Target::Spl, mint_pubkey)?;
    assert_eq!(pausable_config.paused, true.into());

    // Resume the mint with multisig
    app.token_2022_try_resume_multisig(
        Target::Spl,
        mint_pubkey,
        &multisig_pubkey,
        &[AppUser::Admin],
    )?;

    // Check that mint is resumed
    let pausable_config = app.token_2022_query_pausable_config(Target::Spl, mint_pubkey)?;
    assert_eq!(pausable_config.paused, false.into());

    Ok(())
}

#[test]
fn pause_and_resume_multisig_proxy() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::Pausable]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    // Create multisig
    let (_, multisig_keypair) = app.token_2022_try_create_multisig(AppUser::Admin, None)?;
    let multisig_pubkey = &multisig_keypair.pubkey().to_bytes().into();
    app.token_2022_try_initialize_multisig(
        Target::Spl,
        AppUser::Admin,
        multisig_pubkey,
        1,
        &[AppUser::Admin.pubkey(), AppUser::Alice.pubkey()],
    )?;

    // Initialize pausable extension with multisig authority via proxy
    app.token_2022_try_initialize_pausable(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        &multisig_pubkey,
    )?;

    // Initialize the mint via proxy
    app.token_2022_try_initialize_mint(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    // Initially not paused
    let pausable_config = app.token_2022_query_pausable_config(Target::Proxy, mint_pubkey)?;
    assert_eq!(pausable_config.paused, false.into());

    // Pause the mint with multisig via proxy
    app.token_2022_try_pause_multisig(
        Target::Proxy,
        mint_pubkey,
        &multisig_pubkey,
        &[AppUser::Admin],
    )?;

    // Check that mint is paused
    let pausable_config = app.token_2022_query_pausable_config(Target::Proxy, mint_pubkey)?;
    assert_eq!(pausable_config.paused, true.into());

    // Resume the mint with multisig via proxy
    app.token_2022_try_resume_multisig(
        Target::Proxy,
        mint_pubkey,
        &multisig_pubkey,
        &[AppUser::Admin],
    )?;

    // Check that mint is resumed
    let pausable_config = app.token_2022_query_pausable_config(Target::Proxy, mint_pubkey)?;
    assert_eq!(pausable_config.paused, false.into());

    Ok(())
}
