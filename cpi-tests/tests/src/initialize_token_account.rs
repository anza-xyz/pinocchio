use {
    crate::helpers::{
        extensions::token_2022::initialize_token_account::Token2022InitializeAccountExtension,
        suite::{
            core::App,
            types::{AppUser, PinPubkey, Target, TestResult},
        },
    },
    pretty_assertions::assert_eq,
    solana_program_pack::IsInitialized,
    solana_signer::Signer,
    spl_token_2022_interface::extension::ExtensionType,
};

#[test]
pub fn create_and_initialize_token_account() -> TestResult<()> {
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

    let token_account_data = app.token_2022_query_token_account(Target::Spl, &token_account)?;

    //// Assertions
    assert!(token_account_data.is_initialized());
    assert_eq!(token_account_data.mint.to_bytes(), mint);
    assert_eq!(token_account_data.owner.to_bytes(), alice);

    Ok(())
}

#[test]
pub fn proxy_create_and_initialize_token_account() -> TestResult<()> {
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

    let token_account_data = app.token_2022_query_token_account(Target::Proxy, &token_account)?;

    //// Assertions
    assert!(token_account_data.is_initialized());
    assert_eq!(token_account_data.mint.to_bytes(), mint);
    assert_eq!(token_account_data.owner.to_bytes(), alice);

    Ok(())
}
