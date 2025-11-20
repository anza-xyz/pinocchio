use {
    crate::helpers::{
        extensions::token_2022::{
            group_pointer::Token2022GroupPointerExtension,
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
    spl_pod::optional_keys::OptionalNonZeroPubkey,
    spl_token_2022_interface::extension::{group_pointer::GroupPointer, ExtensionType},
};

#[test]
fn initialize_group_pointer_with_default_authority() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::GroupPointer]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let group_pointer = GroupPointer {
        authority: OptionalNonZeroPubkey::default(),
        group_address: OptionalNonZeroPubkey(pin_pubkey_to_addr(mint_pubkey)),
    };

    app.token_2022_try_initialize_group_pointer(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        None,
        Some(mint_pubkey),
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
        app.token_2022_query_group_pointer(Target::Spl, mint_pubkey)?,
        group_pointer
    );

    Ok(())
}

#[test]
fn proxy_initialize_group_pointer_with_default_authority() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::GroupPointer]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let group_pointer = GroupPointer {
        authority: OptionalNonZeroPubkey::default(),
        group_address: OptionalNonZeroPubkey(pin_pubkey_to_addr(mint_pubkey)),
    };

    app.token_2022_try_initialize_group_pointer(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        None,
        Some(mint_pubkey),
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
        app.token_2022_query_group_pointer(Target::Spl, mint_pubkey)?,
        group_pointer
    );
    assert_eq!(
        app.token_2022_query_group_pointer(Target::Proxy, mint_pubkey)?,
        group_pointer
    );

    Ok(())
}

#[test]
fn initialize_and_update_group_pointer() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::GroupPointer]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let group_pointer = GroupPointer {
        authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&mint_authority.pubkey())),
        group_address: OptionalNonZeroPubkey(pin_pubkey_to_addr(mint_pubkey)),
    };

    app.token_2022_try_initialize_group_pointer(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        Some(&mint_authority.pubkey()),
        Some(mint_pubkey),
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
        app.token_2022_query_group_pointer(Target::Spl, mint_pubkey)?,
        group_pointer
    );

    app.token_2022_try_update_group_pointer(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        &mint_authority.pubkey(),
        None,
    )?;

    assert_eq!(
        app.token_2022_query_group_pointer(Target::Spl, mint_pubkey)?,
        GroupPointer {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&mint_authority.pubkey())),
            group_address: OptionalNonZeroPubkey::default(),
        }
    );

    Ok(())
}

#[test]
fn proxy_initialize_and_update_group_pointer() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::GroupPointer]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let group_pointer = GroupPointer {
        authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&mint_authority.pubkey())),
        group_address: OptionalNonZeroPubkey(pin_pubkey_to_addr(mint_pubkey)),
    };

    app.token_2022_try_initialize_group_pointer(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        Some(&mint_authority.pubkey()),
        Some(mint_pubkey),
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
    app.token_2022_try_initialize_group_pointer(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        Some(&mint_authority.pubkey()),
        Some(mint_pubkey),
    )?;

    assert_eq!(
        app.token_2022_query_group_pointer(Target::Spl, mint_pubkey)?,
        group_pointer
    );
    assert_eq!(
        app.token_2022_query_group_pointer(Target::Proxy, mint_pubkey)?,
        group_pointer
    );

    app.token_2022_try_update_group_pointer(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        &mint_authority.pubkey(),
        None,
    )?;

    assert_eq!(
        app.token_2022_query_group_pointer(Target::Spl, mint_pubkey)?,
        GroupPointer {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&mint_authority.pubkey())),
            group_address: OptionalNonZeroPubkey::default(),
        }
    );

    assert_eq!(
        app.token_2022_query_group_pointer(Target::Proxy, mint_pubkey)?,
        GroupPointer {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&mint_authority.pubkey())),
            group_address: OptionalNonZeroPubkey::default(),
        }
    );

    Ok(())
}

#[test]
fn proxy_initialize_and_update_group_pointer_with_multisig() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::GroupPointer]),
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
    let initial_group_address = mint_pubkey;

    // initialize group pointer with multisig authority
    app.token_2022_try_initialize_group_pointer(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        Some(&multisig_authority),
        Some(initial_group_address),
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
        app.token_2022_query_group_pointer(Target::Proxy, mint_pubkey)?,
        GroupPointer {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&multisig_authority)),
            group_address: OptionalNonZeroPubkey(pin_pubkey_to_addr(initial_group_address)),
        }
    );

    // create a new group address for the update
    let (_, new_group_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::TokenGroup]),
    )?;
    let new_group_address = &new_group_keypair.pubkey().to_bytes();

    // update group pointer with sufficient multisig signers (signer1 + signer2)
    app.token_2022_try_update_group_pointer_multisig(
        Target::Proxy,
        mint_pubkey,
        &multisig_authority,
        &[signer1, signer2],
        Some(new_group_address),
    )?;

    assert_eq!(
        app.token_2022_query_group_pointer(Target::Proxy, mint_pubkey)?,
        GroupPointer {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&multisig_authority)),
            group_address: OptionalNonZeroPubkey(pin_pubkey_to_addr(new_group_address)),
        }
    );

    // update to remove group address
    app.token_2022_try_update_group_pointer_multisig(
        Target::Proxy,
        mint_pubkey,
        &multisig_authority,
        &[signer2, signer3],
        None,
    )?;

    assert_eq!(
        app.token_2022_query_group_pointer(Target::Proxy, mint_pubkey)?,
        GroupPointer {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&multisig_authority)),
            group_address: OptionalNonZeroPubkey::default(),
        }
    );

    Ok(())
}
