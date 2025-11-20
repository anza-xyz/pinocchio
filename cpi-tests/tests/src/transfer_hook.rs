use {
    crate::helpers::{
        extensions::token_2022::{
            initialize_mint::Token2022InitializeMintExtension,
            initialize_multisig::Token2022InitializeMultisigExtension,
            transfer_hook::Token2022TransferHookExtension,
        },
        suite::{
            core::App,
            types::{pin_pubkey_to_addr, AppUser, PinPubkey, Target, TestResult},
        },
    },
    pretty_assertions::assert_eq,
    solana_signer::Signer,
    spl_pod::optional_keys::OptionalNonZeroPubkey,
    spl_token_2022_interface::extension::{transfer_hook::TransferHook, ExtensionType},
};

#[test]
fn initialize_transfer_hook_with_authority_and_program() -> TestResult<()> {
    let mut app = App::new(false);

    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::TransferHook]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());
    let transfer_hook_authority = &AppUser::Alice.pubkey();
    let transfer_hook_program = &AppUser::Bob.pubkey();

    app.token_2022_try_initialize_transfer_hook(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        Some(transfer_hook_authority),
        Some(transfer_hook_program),
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
        &app.token_2022_query_transfer_hook(Target::Spl, mint_pubkey)?,
        &TransferHook {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(transfer_hook_authority)),
            program_id: OptionalNonZeroPubkey(pin_pubkey_to_addr(transfer_hook_program)),
        }
    );

    Ok(())
}

#[test]
fn initialize_transfer_hook_with_authority_only() -> TestResult<()> {
    let mut app = App::new(false);

    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::TransferHook]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());

    app.token_2022_try_initialize_transfer_hook(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        Some(&AppUser::Alice.pubkey()),
        None,
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
        &app.token_2022_query_transfer_hook(Target::Spl, mint_pubkey)?,
        &TransferHook {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&AppUser::Alice.pubkey())),
            program_id: OptionalNonZeroPubkey::default(),
        }
    );

    Ok(())
}

#[test]
fn update_transfer_hook() -> TestResult<()> {
    let mut app = App::new(false);

    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::TransferHook]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());
    let transfer_hook_authority = &AppUser::Alice.pubkey();
    let initial_program = &AppUser::Bob.pubkey();
    let new_program = &AppUser::Bob.pubkey();

    // Initialize with initial values
    app.token_2022_try_initialize_transfer_hook(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        Some(transfer_hook_authority),
        Some(initial_program),
    )?;

    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    // Update the transfer hook program
    app.token_2022_try_update_transfer_hook(
        Target::Spl,
        AppUser::Alice,
        mint_pubkey,
        transfer_hook_authority,
        Some(new_program),
    )?;

    assert_eq!(
        &app.token_2022_query_transfer_hook(Target::Spl, mint_pubkey)?,
        &TransferHook {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(transfer_hook_authority)),
            program_id: OptionalNonZeroPubkey(pin_pubkey_to_addr(new_program)),
        }
    );

    Ok(())
}

#[test]
fn update_transfer_hook_multisig() -> TestResult<()> {
    let mut app = App::new(false);

    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::TransferHook]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());
    let transfer_hook_authority = &AppUser::Alice.pubkey();
    let initial_program = &AppUser::Bob.pubkey();
    let new_program = &AppUser::Bob.pubkey();

    // Create multisig
    let (_, multisig_keypair) = app.token_2022_try_create_multisig(AppUser::Admin, None)?;
    let multisig_pubkey = &multisig_keypair.pubkey().to_bytes().into();
    app.token_2022_try_initialize_multisig(
        Target::Spl,
        AppUser::Admin,
        multisig_pubkey,
        2,
        &[AppUser::Alice.pubkey(), AppUser::Bob.pubkey()],
    )?;

    let multisig_authority = multisig_keypair.pubkey().to_bytes().into();

    app.token_2022_try_initialize_transfer_hook(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        Some(&multisig_authority),
        Some(initial_program),
    )?;

    app.token_2022_try_initialize_mint(
        Target::Spl,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &multisig_authority,
        freeze_authority.as_ref(),
    )?;

    // Update using multisig with signers
    app.token_2022_try_update_transfer_hook_multisig(
        Target::Spl,
        mint_pubkey,
        &multisig_authority,
        &[AppUser::Alice, AppUser::Bob],
        Some(new_program),
    )?;

    assert_eq!(
        &app.token_2022_query_transfer_hook(Target::Spl, mint_pubkey)?,
        &TransferHook {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&multisig_authority)),
            program_id: OptionalNonZeroPubkey(pin_pubkey_to_addr(new_program)),
        }
    );

    Ok(())
}

#[test]
fn initialize_transfer_hook_proxy() -> TestResult<()> {
    let mut app = App::new(false);

    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::TransferHook]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());
    let transfer_hook_authority = &AppUser::Alice.pubkey();
    let transfer_hook_program = &AppUser::Bob.pubkey();

    app.token_2022_try_initialize_transfer_hook(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        Some(transfer_hook_authority),
        Some(transfer_hook_program),
    )?;

    app.token_2022_try_initialize_mint(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    assert_eq!(
        &app.token_2022_query_transfer_hook(Target::Proxy, mint_pubkey)?,
        &TransferHook {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(transfer_hook_authority)),
            program_id: OptionalNonZeroPubkey(pin_pubkey_to_addr(transfer_hook_program)),
        }
    );

    Ok(())
}

#[test]
fn update_transfer_hook_proxy() -> TestResult<()> {
    let mut app = App::new(false);

    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::TransferHook]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());
    let transfer_hook_authority = &AppUser::Alice.pubkey();
    let initial_program = &AppUser::Bob.pubkey();
    let new_program = &AppUser::Bob.pubkey();

    // Initialize with initial values
    app.token_2022_try_initialize_transfer_hook(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        Some(transfer_hook_authority),
        Some(initial_program),
    )?;

    app.token_2022_try_initialize_mint(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &mint_authority.pubkey(),
        freeze_authority.as_ref(),
    )?;

    // Update the transfer hook program
    app.token_2022_try_update_transfer_hook(
        Target::Proxy,
        AppUser::Alice,
        mint_pubkey,
        transfer_hook_authority,
        Some(new_program),
    )?;

    assert_eq!(
        &app.token_2022_query_transfer_hook(Target::Proxy, mint_pubkey)?,
        &TransferHook {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(transfer_hook_authority)),
            program_id: OptionalNonZeroPubkey(pin_pubkey_to_addr(new_program)),
        }
    );

    Ok(())
}

#[test]
fn update_transfer_hook_multisig_proxy() -> TestResult<()> {
    let mut app = App::new(false);

    let (_, mint_keypair) = app.token_2022_try_create_mint_account(
        AppUser::Admin,
        None,
        Some(&[ExtensionType::TransferHook]),
    )?;

    let mint_pubkey = &mint_keypair.pubkey().to_bytes();
    let decimals: u8 = 6;
    let mint_authority = AppUser::Admin;
    let freeze_authority = Some(AppUser::Admin.pubkey());
    let transfer_hook_authority = &AppUser::Alice.pubkey();
    let initial_program = &AppUser::Bob.pubkey();
    let new_program = &AppUser::Bob.pubkey();

    // Create multisig
    let (_, multisig_keypair) = app.token_2022_try_create_multisig(AppUser::Admin, None)?;
    let multisig_pubkey = &multisig_keypair.pubkey().to_bytes().into();
    app.token_2022_try_initialize_multisig(
        Target::Spl,
        AppUser::Admin,
        multisig_pubkey,
        2,
        &[AppUser::Alice.pubkey(), AppUser::Bob.pubkey()],
    )?;

    let multisig_authority = multisig_keypair.pubkey().to_bytes().into();

    app.token_2022_try_initialize_transfer_hook(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        Some(&multisig_authority),
        Some(initial_program),
    )?;

    app.token_2022_try_initialize_mint(
        Target::Proxy,
        AppUser::Admin,
        mint_pubkey,
        decimals,
        &multisig_authority,
        freeze_authority.as_ref(),
    )?;

    // Update using multisig with signers
    app.token_2022_try_update_transfer_hook_multisig(
        Target::Proxy,
        mint_pubkey,
        &multisig_authority,
        &[AppUser::Alice, AppUser::Bob],
        Some(new_program),
    )?;

    assert_eq!(
        &app.token_2022_query_transfer_hook(Target::Proxy, mint_pubkey)?,
        &TransferHook {
            authority: OptionalNonZeroPubkey(pin_pubkey_to_addr(&multisig_authority)),
            program_id: OptionalNonZeroPubkey(pin_pubkey_to_addr(new_program)),
        }
    );

    Ok(())
}
