use {
    crate::helpers::{
        extensions::token_2022::{
            default_account_state::Token2022DefaultAccountStateExtension,
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
    spl_token_2022_interface::{
        extension::{default_account_state::DefaultAccountState, ExtensionType},
        state::AccountState,
    },
};

#[test]
fn initialize_default_account_state() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::DefaultAccountState]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let default_state = AccountState::Frozen;
    let default_account_state_config = DefaultAccountState {
        state: default_state.into(),
    };

    app.token_2022_try_initialize_default_account_state(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        default_state,
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
        app.token_2022_query_default_account_state(Target::Spl, mint_pubkey)?,
        default_account_state_config
    );

    Ok(())
}

#[test]
fn proxy_initialize_default_account_state() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::DefaultAccountState]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let default_state = AccountState::Frozen;
    let default_account_state_config = DefaultAccountState {
        state: default_state.into(),
    };

    app.token_2022_try_initialize_default_account_state(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        default_state,
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
        app.token_2022_query_default_account_state(Target::Proxy, mint_pubkey)?,
        default_account_state_config
    );

    assert_eq!(
        app.token_2022_query_default_account_state(Target::Spl, mint_pubkey)?,
        default_account_state_config
    );

    Ok(())
}

#[test]
fn initialize_and_update_default_account_state() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::DefaultAccountState]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    // Initialize with Frozen state
    let initial_state = AccountState::Frozen;
    app.token_2022_try_initialize_default_account_state(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        initial_state,
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
    let initial_config = DefaultAccountState {
        state: initial_state.into(),
    };

    assert_eq!(
        app.token_2022_query_default_account_state(Target::Spl, mint_pubkey)?,
        initial_config
    );

    // Update to Initialized state
    let updated_state = AccountState::Initialized;
    app.token_2022_try_update_default_account_state(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        &freeze_authority.unwrap(),
        updated_state,
    )?;

    // Verify updated state
    let updated_config = DefaultAccountState {
        state: updated_state.into(),
    };

    assert_eq!(
        app.token_2022_query_default_account_state(Target::Spl, mint_pubkey)?,
        updated_config
    );

    Ok(())
}

#[test]
fn proxy_initialize_and_update_default_account_state() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::DefaultAccountState]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    // Initialize with Frozen state
    let initial_state = AccountState::Frozen;
    app.token_2022_try_initialize_default_account_state(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        initial_state,
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
    let initial_config = DefaultAccountState {
        state: initial_state.into(),
    };

    assert_eq!(
        app.token_2022_query_default_account_state(Target::Proxy, mint_pubkey)?,
        initial_config
    );

    assert_eq!(
        app.token_2022_query_default_account_state(Target::Spl, mint_pubkey)?,
        initial_config
    );

    // Update to Initialized state
    let updated_state = AccountState::Initialized;
    app.token_2022_try_update_default_account_state(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        &freeze_authority.unwrap(),
        updated_state,
    )?;

    // Verify updated state
    let updated_config = DefaultAccountState {
        state: updated_state.into(),
    };

    assert_eq!(
        app.token_2022_query_default_account_state(Target::Proxy, mint_pubkey)?,
        updated_config
    );

    assert_eq!(
        app.token_2022_query_default_account_state(Target::Spl, mint_pubkey)?,
        updated_config
    );

    Ok(())
}

#[test]
fn initialize_and_update_default_account_state_multisig() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::DefaultAccountState]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;

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
        &[signer1.pubkey(), signer2.pubkey(), signer3.pubkey()],
    )?;

    let multisig_freeze_authority = multisig_kp.pubkey().to_bytes().into();
    let freeze_authority = Some(multisig_kp.pubkey().to_bytes());
    let signers = &[signer1, signer2];

    // Initialize with Frozen state
    let initial_state = AccountState::Frozen;
    app.token_2022_try_initialize_default_account_state(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        initial_state,
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
    let initial_config = DefaultAccountState {
        state: initial_state.into(),
    };

    assert_eq!(
        app.token_2022_query_default_account_state(Target::Spl, mint_pubkey)?,
        initial_config
    );

    // Update to Initialized state using multisig
    let updated_state = AccountState::Initialized;
    app.token_2022_try_update_default_account_state_multisig(
        Target::Spl,
        mint_pubkey,
        &multisig_freeze_authority,
        signers,
        updated_state,
    )?;

    // Verify updated state
    let updated_config = DefaultAccountState {
        state: updated_state.into(),
    };

    assert_eq!(
        app.token_2022_query_default_account_state(Target::Spl, mint_pubkey)?,
        updated_config
    );

    Ok(())
}

#[test]
fn proxy_initialize_and_update_default_account_state_multisig() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::DefaultAccountState]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;

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
        &[signer1.pubkey(), signer2.pubkey(), signer3.pubkey()],
    )?;

    let multisig_freeze_authority = multisig_kp.pubkey().to_bytes().into();
    let freeze_authority = Some(multisig_kp.pubkey().to_bytes());
    let signers = &[signer1, signer2];

    // Initialize with Frozen state
    let initial_state = AccountState::Frozen;
    app.token_2022_try_initialize_default_account_state(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        initial_state,
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
    let initial_config = DefaultAccountState {
        state: initial_state.into(),
    };

    assert_eq!(
        app.token_2022_query_default_account_state(Target::Proxy, mint_pubkey)?,
        initial_config
    );

    assert_eq!(
        app.token_2022_query_default_account_state(Target::Spl, mint_pubkey)?,
        initial_config
    );

    // Update to Initialized state using multisig
    let updated_state = AccountState::Initialized;
    app.token_2022_try_update_default_account_state_multisig(
        Target::Proxy,
        mint_pubkey,
        &multisig_freeze_authority,
        signers,
        updated_state,
    )?;

    // Verify updated state
    let updated_config = DefaultAccountState {
        state: updated_state.into(),
    };

    assert_eq!(
        app.token_2022_query_default_account_state(Target::Proxy, mint_pubkey)?,
        updated_config
    );

    assert_eq!(
        app.token_2022_query_default_account_state(Target::Spl, mint_pubkey)?,
        updated_config
    );

    Ok(())
}
