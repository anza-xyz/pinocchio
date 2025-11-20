use {
    crate::helpers::{
        extensions::token_2022::{
            group_member_pointer::Token2022GroupMemberPointerExtension,
            initialize_mint::Token2022InitializeMintExtension,
            initialize_multisig::Token2022InitializeMultisigExtension,
        },
        suite::{
            core::App,
            types::{to_optional_non_zero_pubkey, AppUser, PinPubkey, Target, TestResult},
        },
    },
    pretty_assertions::assert_eq,
    solana_signer::Signer,
    spl_token_2022_interface::extension::ExtensionType,
};

#[test]
fn initialize_and_update_group_member_pointer() -> TestResult<()> {
    let mut app = App::new(false);

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

    assert_eq!(
        &app.token_2022_query_group_member_pointer(Target::Spl, mint_b)
            .map(|x| x.member_address.0.to_bytes())?,
        mint_b
    );

    app.token_2022_try_update_group_member_pointer(
        Target::Spl,
        AppUser::Admin,
        mint_b,
        &AppUser::Admin.pubkey(),
        None,
    )?;

    assert_eq!(
        app.token_2022_query_group_member_pointer(Target::Spl, mint_b)
            .map(|x| x.member_address)?,
        to_optional_non_zero_pubkey(None)
    );

    Ok(())
}

#[test]
fn proxy_initialize_and_update_group_member_pointer() -> TestResult<()> {
    let mut app = App::new(false);

    let (_, mint_b_kp) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::GroupMemberPointer]),
    )?;
    let mint_b = &mint_b_kp.pubkey().to_bytes();

    // 2nd to run internal checks
    for _ in [0..=1] {
        app.token_2022_try_initialize_group_member_pointer(
            Target::Proxy,
            AppUser::Admin,
            mint_b,
            Some(&AppUser::Admin.pubkey()),
            Some(mint_b),
        )?;
    }

    app.token_2022_try_initialize_mint(
        Target::Proxy,
        AppUser::Admin,
        mint_b,
        6,
        &AppUser::Admin.pubkey(),
        Some(&AppUser::Admin.pubkey()),
    )?;

    assert_eq!(
        &app.token_2022_query_group_member_pointer(Target::Proxy, mint_b)
            .map(|x| x.member_address.0.to_bytes())?,
        mint_b
    );

    app.token_2022_try_update_group_member_pointer(
        Target::Proxy,
        AppUser::Admin,
        mint_b,
        &AppUser::Admin.pubkey(),
        None,
    )?;

    assert_eq!(
        app.token_2022_query_group_member_pointer(Target::Proxy, mint_b)
            .map(|x| x.member_address)?,
        to_optional_non_zero_pubkey(None)
    );

    Ok(())
}

#[test]
fn proxy_initialize_and_update_group_member_pointer_with_multisig() -> TestResult<()> {
    let mut app = App::new(false);
    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::GroupMemberPointer]),
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

    // initial group member pointer configuration with multisig authority
    let initial_member_address = mint_pubkey;

    // initialize group member pointer with multisig authority
    app.token_2022_try_initialize_group_member_pointer(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        Some(&multisig_authority),
        Some(initial_member_address),
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
        &app.token_2022_query_group_member_pointer(Target::Proxy, mint_pubkey)
            .map(|x| x.member_address.0.to_bytes())?,
        initial_member_address
    );

    // create a new member address for the update
    let (_, new_member_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::TokenGroupMember]),
    )?;
    let new_member_address = &new_member_keypair.pubkey().to_bytes();

    // update group member pointer with sufficient multisig signers (signer1 + signer2)
    app.token_2022_try_update_group_member_pointer_multisig(
        Target::Proxy,
        mint_pubkey,
        &multisig_authority,
        &[signer1, signer2],
        Some(new_member_address),
    )?;

    assert_eq!(
        &app.token_2022_query_group_member_pointer(Target::Proxy, mint_pubkey)
            .map(|x| x.member_address.0.to_bytes())?,
        new_member_address
    );

    // update to remove member address
    app.token_2022_try_update_group_member_pointer_multisig(
        Target::Proxy,
        mint_pubkey,
        &multisig_authority,
        &[signer2, signer3],
        None,
    )?;

    assert_eq!(
        app.token_2022_query_group_member_pointer(Target::Proxy, mint_pubkey)
            .map(|x| x.member_address)?,
        to_optional_non_zero_pubkey(None)
    );

    Ok(())
}
