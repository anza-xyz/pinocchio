use {
    crate::helpers::{
        extensions::token_2022::initialize_mint::Token2022InitializeMintExtension,
        suite::{
            core::App,
            types::{pin_pubkey_to_addr, to_c_option, AppUser, PinPubkey, TestResult},
        },
    },
    pretty_assertions::assert_eq,
    solana_program_option::COption,
    solana_signer::Signer,
};

#[test]
fn initialize_mint() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(AppUser::Admin, None, None)?;

    let mint = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Alice.pubkey();
    let freeze_authority = Some(AppUser::Bob.pubkey());
    let mint_state = spl_token_2022_interface::state::Mint {
        mint_authority: COption::Some(mint_authority.into()),
        supply: 0,
        decimals,
        is_initialized: true,
        freeze_authority: to_c_option(freeze_authority.as_ref().map(pin_pubkey_to_addr)),
    };

    app.token_2022_try_initialize_mint(
        AppUser::Admin,
        mint,
        decimals,
        &mint_authority,
        freeze_authority.as_ref(),
    )?;
    assert_eq!(app.token_2022_query_mint_state(mint)?, mint_state);

    Ok(())
}

#[test]
fn proxy_initialize_mint() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(AppUser::Admin, None, None)?;

    let mint = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Alice.pubkey();
    let freeze_authority = Some(AppUser::Bob.pubkey());
    let mint_state = spl_token_2022_interface::state::Mint {
        mint_authority: COption::Some(mint_authority.into()),
        supply: 0,
        decimals,
        is_initialized: true,
        freeze_authority: to_c_option(freeze_authority.as_ref().map(pin_pubkey_to_addr)),
    };

    // 1st to initialize mint, 2nd to run internal checks
    for _ in [0..=1] {
        app.token_2022_proxy_try_initialize_mint(
            AppUser::Admin,
            mint,
            decimals,
            &mint_authority,
            freeze_authority.as_ref(),
        )?;
    }
    assert_eq!(app.token_2022_proxy_query_mint_state(mint)?, mint_state);
    assert_eq!(app.token_2022_query_mint_state(mint)?, mint_state);

    Ok(())
}
