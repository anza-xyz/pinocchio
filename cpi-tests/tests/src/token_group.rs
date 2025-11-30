use {
    crate::helpers::{
        extensions::token_2022::{
            group_pointer::Token2022GroupPointerExtension,
            initialize_mint::Token2022InitializeMintExtension,
            token_group::Token2022TokenGroupExtension,
        },
        suite::{
            core::App,
            types::{pin_pubkey_to_addr, AppUser, PinPubkey, Target, TestResult},
        },
    },
    pretty_assertions::assert_eq,
    solana_signer::Signer,
    spl_pod::optional_keys::OptionalNonZeroPubkey,
    spl_token_2022_interface::extension::ExtensionType,
};

#[test]
fn initialize_token_group() -> TestResult<()> {
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

    let update_authority = pin_pubkey_to_addr(&AppUser::Admin.pubkey());
    let max_size = 10;
    let token_group = spl_token_group_interface::state::TokenGroup::new(
        &pin_pubkey_to_addr(mint_pubkey),
        OptionalNonZeroPubkey(update_authority),
        max_size,
    );

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

    app.token_2022_try_initialize_token_group(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        mint_pubkey,
        mint_authority,
        Some(&update_authority.to_bytes()),
        max_size,
    )?;

    assert_eq!(
        app.token_2022_query_token_group(Target::Spl, mint_pubkey)?,
        token_group
    );

    app.token_2022_try_update_group_max_size(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        mint_authority,
        &update_authority.to_bytes(),
        max_size,
    )?;

    app.token_2022_try_update_group_authority(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        &mint_authority.pubkey(),
        None,
    )?;

    Ok(())
}

#[test]
fn proxy_initialize_token_group() -> TestResult<()> {
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

    let update_authority = pin_pubkey_to_addr(&AppUser::Admin.pubkey());
    let max_size = 10;
    let token_group = spl_token_group_interface::state::TokenGroup::new(
        &pin_pubkey_to_addr(mint_pubkey),
        OptionalNonZeroPubkey(update_authority),
        max_size,
    );

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

    // 2nd to run internal checks
    for _ in [0..=1] {
        app.token_2022_try_initialize_token_group(
            Target::Proxy,
            AppUser::Admin,
            mint_pubkey,
            mint_pubkey,
            mint_authority,
            Some(&update_authority.to_bytes()),
            max_size,
        )?;
    }

    assert_eq!(
        app.token_2022_query_token_group(Target::Spl, mint_pubkey)?,
        token_group
    );
    assert_eq!(
        app.token_2022_query_token_group(Target::Proxy, mint_pubkey)?,
        token_group
    );

    app.token_2022_try_update_group_max_size(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        mint_authority,
        &update_authority.to_bytes(),
        max_size,
    )?;

    app.token_2022_try_update_group_authority(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        &mint_authority.pubkey(),
        None,
    )?;

    Ok(())
}
