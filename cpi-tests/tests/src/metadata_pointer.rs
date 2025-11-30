use {
    crate::helpers::{
        extensions::token_2022::{
            initialize_mint::Token2022InitializeMintExtension,
            initialize_multisig::Token2022InitializeMultisigExtension,
            metadata_pointer::Token2022MetadataPointerExtension,
        },
        suite::{
            core::App,
            types::{pin_pubkey_to_addr, AppUser, PinPubkey, Target, TestResult},
        },
    },
    pretty_assertions::assert_eq,
    solana_signer::Signer,
    spl_pod::optional_keys::OptionalNonZeroPubkey,
    spl_token_2022_interface::extension::{metadata_pointer::MetadataPointer, ExtensionType},
};

#[test]
fn initialize_metadata_pointer_with_default_authority() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::MetadataPointer]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let metadata_pointer = MetadataPointer {
        authority: OptionalNonZeroPubkey::default(),
        metadata_address: OptionalNonZeroPubkey(pin_pubkey_to_addr(mint_pubkey)),
    };

    app.token_2022_try_initialize_metadata_pointer(
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
        app.token_2022_query_metadata_pointer(Target::Spl, mint_pubkey)?,
        metadata_pointer
    );

    Ok(())
}

#[test]
fn proxy_initialize_metadata_pointer_with_default_authority() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::MetadataPointer]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let metadata_pointer = MetadataPointer {
        authority: OptionalNonZeroPubkey::default(),
        metadata_address: OptionalNonZeroPubkey(pin_pubkey_to_addr(mint_pubkey)),
    };

    app.token_2022_try_initialize_metadata_pointer(
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
        app.token_2022_query_metadata_pointer(Target::Spl, mint_pubkey)?,
        metadata_pointer
    );
    assert_eq!(
        app.token_2022_query_metadata_pointer(Target::Proxy, mint_pubkey)?,
        metadata_pointer
    );

    Ok(())
}

#[test]
fn initialize_and_update_metadata_pointer() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::MetadataPointer]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let metadata_pointer = MetadataPointer {
        authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&mint_authority.pubkey())),
        metadata_address: OptionalNonZeroPubkey(pin_pubkey_to_addr(mint_pubkey)),
    };

    app.token_2022_try_initialize_metadata_pointer(
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
        app.token_2022_query_metadata_pointer(Target::Spl, mint_pubkey)?,
        metadata_pointer
    );

    app.token_2022_try_update_metadata_pointer(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        &mint_authority.pubkey(),
        None,
    )?;

    assert_eq!(
        app.token_2022_query_metadata_pointer(Target::Spl, mint_pubkey)?,
        MetadataPointer {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&mint_authority.pubkey())),
            metadata_address: OptionalNonZeroPubkey::default(),
        }
    );

    Ok(())
}

#[test]
fn proxy_initialize_and_update_metadata_pointer() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::MetadataPointer]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    let metadata_pointer = MetadataPointer {
        authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&mint_authority.pubkey())),
        metadata_address: OptionalNonZeroPubkey(pin_pubkey_to_addr(mint_pubkey)),
    };

    app.token_2022_try_initialize_metadata_pointer(
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
    app.token_2022_try_initialize_metadata_pointer(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        Some(&mint_authority.pubkey()),
        Some(mint_pubkey),
    )?;

    assert_eq!(
        app.token_2022_query_metadata_pointer(Target::Spl, mint_pubkey)?,
        metadata_pointer
    );
    assert_eq!(
        app.token_2022_query_metadata_pointer(Target::Proxy, mint_pubkey)?,
        metadata_pointer
    );

    app.token_2022_try_update_metadata_pointer(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        &mint_authority.pubkey(),
        None,
    )?;

    assert_eq!(
        app.token_2022_query_metadata_pointer(Target::Spl, mint_pubkey)?,
        MetadataPointer {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&mint_authority.pubkey())),
            metadata_address: OptionalNonZeroPubkey::default(),
        }
    );

    assert_eq!(
        app.token_2022_query_metadata_pointer(Target::Proxy, mint_pubkey)?,
        MetadataPointer {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&mint_authority.pubkey())),
            metadata_address: OptionalNonZeroPubkey::default(),
        }
    );

    Ok(())
}

#[test]
fn proxy_initialize_and_update_metadata_pointer_with_multisig() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::MetadataPointer]),
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
    let initial_metadata_address = mint_pubkey;

    // initialize metadata pointer with multisig authority
    app.token_2022_try_initialize_metadata_pointer(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        Some(&multisig_authority),
        Some(initial_metadata_address),
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
        app.token_2022_query_metadata_pointer(Target::Proxy, mint_pubkey)?,
        MetadataPointer {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&multisig_authority)),
            metadata_address: OptionalNonZeroPubkey(pin_pubkey_to_addr(initial_metadata_address)),
        }
    );

    // create a new metadata address for the update
    let (_, new_metadata_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::MetadataPointer]),
    )?;
    let new_metadata_address = &new_metadata_keypair.pubkey().to_bytes();

    // update metadata pointer with sufficient multisig signers (signer1 + signer2)
    app.token_2022_try_update_metadata_pointer_multisig(
        Target::Proxy,
        mint_pubkey,
        &multisig_authority,
        &[signer1, signer2],
        Some(new_metadata_address),
    )?;

    assert_eq!(
        app.token_2022_query_metadata_pointer(Target::Proxy, mint_pubkey)?,
        MetadataPointer {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&multisig_authority)),
            metadata_address: OptionalNonZeroPubkey(pin_pubkey_to_addr(new_metadata_address)),
        }
    );

    // update to remove metadata address
    app.token_2022_try_update_metadata_pointer_multisig(
        Target::Proxy,
        mint_pubkey,
        &multisig_authority,
        &[signer2, signer3],
        None,
    )?;

    assert_eq!(
        app.token_2022_query_metadata_pointer(Target::Proxy, mint_pubkey)?,
        MetadataPointer {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&multisig_authority)),
            metadata_address: OptionalNonZeroPubkey::default(),
        }
    );

    Ok(())
}
