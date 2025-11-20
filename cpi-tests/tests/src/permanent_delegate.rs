use {
    crate::helpers::{
        extensions::token_2022::{
            initialize_mint::Token2022InitializeMintExtension,
            permanent_delegate::Token2022PermanentDelegateExtension,
        },
        suite::{
            core::App,
            types::{AppUser, PinPubkey, Target, TestResult},
        },
    },
    pretty_assertions::assert_eq,
    solana_signer::Signer,
    spl_token_2022_interface::extension::ExtensionType,
};

#[test]
fn initialize_permanent_delegate() -> TestResult<()> {
    let mut app = App::new(false);

    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::PermanentDelegate]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());
    let delegate_pubkey = &AppUser::Alice.pubkey();

    app.token_2022_try_initialize_permanent_delegate(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        delegate_pubkey,
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
        &app.token_2022_query_permanent_delegate(Target::Spl, mint_pubkey)
            .map(|x| x.delegate.0.to_bytes())?,
        delegate_pubkey
    );

    Ok(())
}

#[test]
fn proxy_initialize_permanent_delegate() -> TestResult<()> {
    let mut app = App::new(false);

    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::PermanentDelegate]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());
    let delegate_pubkey = &AppUser::Alice.pubkey();

    app.token_2022_try_initialize_permanent_delegate(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        delegate_pubkey,
    )?;

    app.token_2022_try_initialize_mint(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    // execute 2nd time to run internal checks
    app.token_2022_try_initialize_permanent_delegate(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        delegate_pubkey,
    )?;

    assert_eq!(
        &app.token_2022_query_permanent_delegate(Target::Proxy, mint_pubkey)
            .map(|x| x.delegate.0.to_bytes())?,
        delegate_pubkey
    );

    Ok(())
}
