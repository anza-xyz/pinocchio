use {
    crate::helpers::{
        extensions::token_2022::{
            group_member_pointer::Token2022GroupMemberPointerExtension,
            group_pointer::Token2022GroupPointerExtension,
            initialize_mint::Token2022InitializeMintExtension,
            token_group::Token2022TokenGroupExtension,
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
fn initialize_token_group_member() -> TestResult<()> {
    let mut app = App::new(false);

    // === Create a group ===
    let (_, mint_a_kp) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::GroupPointer]),
    )?;
    let mint_a = &mint_a_kp.pubkey().to_bytes();

    app.token_2022_try_initialize_group_pointer(
        Target::Spl,
        AppUser::Admin,
        mint_a,
        Some(&AppUser::Admin.pubkey()),
        Some(mint_a),
    )?;

    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        mint_a,
        6,
        &AppUser::Admin.pubkey(),
        Some(&AppUser::Admin.pubkey()),
    )?;

    app.token_2022_try_initialize_token_group(
        Target::Spl,
        AppUser::Admin,
        mint_a,
        mint_a,
        AppUser::Admin,
        Some(&AppUser::Admin.pubkey()),
        5,
    )?;

    // === Add a group member ===
    let (_, mint_b_kp) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::GroupMemberPointer]),
    )?;
    let mint_b = &mint_b_kp.pubkey().to_bytes();

    app.token_2022_try_initialize_group_member_pointer(
        Target::Spl,
        AppUser::Admin,
        mint_b,
        Some(&AppUser::Admin.pubkey()),
        Some(mint_b),
    )?;

    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        mint_b,
        6,
        &AppUser::Admin.pubkey(),
        Some(&AppUser::Admin.pubkey()),
    )?;

    app.token_2022_try_initialize_member(
        Target::Spl,
        AppUser::Admin,
        mint_a,
        &AppUser::Admin.keypair(),
        mint_b,
        mint_b,
        &AppUser::Admin.keypair(),
    )?;

    assert_eq!(
        &app.token_2022_query_group_pointer(Target::Spl, mint_a)
            .map(|x| x.group_address.0.to_bytes())?,
        mint_a
    );
    assert_eq!(
        &app.token_2022_query_group_member_pointer(Target::Spl, mint_b)
            .map(|x| x.member_address.0.to_bytes())?,
        mint_b
    );

    assert_eq!(
        &app.token_2022_query_token_group(Target::Spl, mint_a)
            .map(|x| x.mint.to_bytes())?,
        mint_a
    );
    assert_eq!(
        &app.token_2022_query_token_group_member(Target::Spl, mint_b)
            .map(|x| x.group.to_bytes())?,
        mint_a
    );

    Ok(())
}

#[test]
fn proxy_initialize_token_group_member() -> TestResult<()> {
    let mut app = App::new(false);

    // === Create a group ===
    let (_, mint_a_kp) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::GroupPointer]),
    )?;
    let mint_a = &mint_a_kp.pubkey().to_bytes();

    app.token_2022_try_initialize_group_pointer(
        Target::Proxy,
        AppUser::Admin,
        mint_a,
        Some(&AppUser::Admin.pubkey()),
        Some(mint_a),
    )?;

    app.token_2022_try_initialize_mint(
        Target::Proxy,
        AppUser::Admin,
        mint_a,
        6,
        &AppUser::Admin.pubkey(),
        Some(&AppUser::Admin.pubkey()),
    )?;

    app.token_2022_try_initialize_token_group(
        Target::Proxy,
        AppUser::Admin,
        mint_a,
        mint_a,
        AppUser::Admin,
        Some(&AppUser::Admin.pubkey()),
        5,
    )?;

    // === Add a group member ===
    let (_, mint_b_kp) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::GroupMemberPointer]),
    )?;
    let mint_b = &mint_b_kp.pubkey().to_bytes();

    app.token_2022_try_initialize_group_member_pointer(
        Target::Proxy,
        AppUser::Admin,
        mint_b,
        Some(&AppUser::Admin.pubkey()),
        Some(mint_b),
    )?;

    app.token_2022_try_initialize_mint(
        Target::Proxy,
        AppUser::Admin,
        mint_b,
        6,
        &AppUser::Admin.pubkey(),
        Some(&AppUser::Admin.pubkey()),
    )?;

    // 2nd to run internal checks
    for _ in [0..=1] {
        app.token_2022_try_initialize_member(
            Target::Proxy,
            AppUser::Admin,
            mint_a,
            &AppUser::Admin.keypair(),
            mint_b,
            mint_b,
            &AppUser::Admin.keypair(),
        )?;
    }

    assert_eq!(
        &app.token_2022_query_group_pointer(Target::Proxy, mint_a)
            .map(|x| x.group_address.0.to_bytes())?,
        mint_a
    );
    assert_eq!(
        &app.token_2022_query_group_member_pointer(Target::Proxy, mint_b)
            .map(|x| x.member_address.0.to_bytes())?,
        mint_b
    );

    assert_eq!(
        &app.token_2022_query_token_group(Target::Proxy, mint_a)
            .map(|x| x.mint.to_bytes())?,
        mint_a
    );
    assert_eq!(
        &app.token_2022_query_token_group_member(Target::Proxy, mint_b)
            .map(|x| x.group.to_bytes())?,
        mint_a
    );

    Ok(())
}
