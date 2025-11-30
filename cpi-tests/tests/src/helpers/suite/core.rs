use {
    crate::helpers::suite::{
        solana_kite::{
            create_associated_token_account, create_token_mint, deploy_program,
            get_token_account_balance, mint_tokens_to_account,
        },
        types::{
            addr_to_sol_pubkey, pin_to_sol_pubkey, AppAsset, AppCoin, AppToken, AppUser,
            GetDecimals, SolPubkey, TestError, TestResult,
        },
    },
    litesvm::{types::TransactionMetadata, LiteSVM},
    solana_compute_budget::compute_budget::ComputeBudget,
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::Keypair,
    solana_program::native_token::LAMPORTS_PER_SOL,
    solana_pubkey::Pubkey,
    solana_signer::{signers::Signers, Signer},
    solana_system_interface,
    solana_transaction::Transaction,
    spl_associated_token_account::get_associated_token_address,
    strum::IntoEnumIterator,
};

pub const PROGRAM_NAME_TOKEN_2022_PROXY: &str = "token_2022_proxy";

pub struct ProgramId {
    // 3rd party
    pub system_program: Pubkey,
    pub token_program: Pubkey,
    pub token_2022_program: Pubkey,
    pub associated_token_program: Pubkey,

    // custom
    pub token_2022_proxy: Pubkey,
}

pub struct App {
    pub litesvm: LiteSVM,
    pub is_log_displayed: bool,

    pub program_id: ProgramId,
}

impl App {
    pub fn create_app_with_programs(is_log_displayed: bool) -> Self {
        // prepare environment with balances
        let mut litesvm = Self::init_env_with_balances();

        // specify programs
        let program_id = ProgramId {
            // 3rd party
            system_program: addr_to_sol_pubkey(&solana_system_interface::program::ID),
            token_program: spl_token::ID,
            token_2022_program: addr_to_sol_pubkey(&spl_token_2022_interface::ID),
            associated_token_program: spl_associated_token_account::ID,

            // custom
            token_2022_proxy: token_2022_proxy::ID.into(),
        };

        // upload custom programs
        upload_program(
            &mut litesvm,
            PROGRAM_NAME_TOKEN_2022_PROXY,
            &program_id.token_2022_proxy,
        );

        Self {
            litesvm,
            is_log_displayed,

            program_id,
        }
    }

    pub fn new(is_log_displayed: bool) -> Self {
        Self::create_app_with_programs(is_log_displayed)
    }

    fn init_env_with_balances() -> LiteSVM {
        let mut litesvm = LiteSVM::new().with_compute_budget(ComputeBudget {
            compute_unit_limit: 10_000_000,
            ..ComputeBudget::default()
        });

        // airdrop SOL
        for user in AppUser::iter() {
            litesvm
                .airdrop(
                    &user.pubkey(),
                    user.get_initial_asset_amount(AppCoin::SOL) * LAMPORTS_PER_SOL,
                )
                .unwrap();
        }

        // create tokens
        for token in AppToken::iter() {
            // skip WSOL
            if token == AppToken::WSOL {
                continue;
            }

            create_token_mint(
                &mut litesvm,
                &AppUser::Admin.keypair(),
                token.get_decimals(),
                Some(token.pubkey()),
            )
            .unwrap();
        }

        // mint tokens
        for user in AppUser::iter() {
            for token in AppToken::iter() {
                // skip WSOL
                if token == AppToken::WSOL {
                    continue;
                }

                let ata = App::create_ata(
                    &mut litesvm,
                    &AppUser::Admin.keypair(),
                    &user.pubkey(),
                    &token.pubkey(),
                )
                .unwrap();

                mint_tokens_to_account(
                    &mut litesvm,
                    &token.pubkey(),
                    &ata,
                    user.get_initial_asset_amount(token) * 10u64.pow(token.get_decimals() as u32),
                    &AppUser::Admin.keypair(),
                )
                .unwrap();
            }
        }

        litesvm
    }

    // utils

    pub fn transfer_sol(
        &mut self,
        sender: AppUser,
        recipient: &Pubkey,
        amount: u64,
    ) -> TestResult<TransactionMetadata> {
        let payer = &sender.pubkey();
        let signers = &[sender.keypair()];
        let ix = solana_system_interface::instruction::transfer(
            &payer.to_bytes().into(),
            &recipient.to_bytes().into(),
            amount,
        );

        let ix_legacy = solana_instruction::Instruction {
            program_id: addr_to_sol_pubkey(&ix.program_id),
            accounts: ix
                .accounts
                .into_iter()
                .map(|x| solana_instruction::AccountMeta {
                    pubkey: addr_to_sol_pubkey(&x.pubkey),
                    is_signer: x.is_signer,
                    is_writable: x.is_writable,
                })
                .collect(),
            data: ix.data,
        };

        extension::send_tx(
            &mut self.litesvm,
            &[ix_legacy],
            signers,
            self.is_log_displayed,
        )
    }

    pub fn get_balance(&self, user: AppUser, asset: impl Into<AppAsset>) -> u64 {
        let address = &user.pubkey();

        match asset.into() {
            AppAsset::Coin(_) => self.get_coin_balance(address),
            AppAsset::Token(mint) => self.get_ata_token_balance(address, &mint.pubkey()),
        }
    }

    pub fn get_coin_balance(&self, address: &Pubkey) -> u64 {
        self.litesvm.get_balance(address).unwrap_or_default()
    }

    pub fn get_ata_token_balance(&self, address: &Pubkey, mint: &Pubkey) -> u64 {
        get_token_account_balance(&self.litesvm, &Self::get_ata(address, mint)).unwrap_or_default()
    }

    pub fn get_pda_token_balance(&self, token_account: &Pubkey) -> u64 {
        get_token_account_balance(&self.litesvm, token_account).unwrap_or_default()
    }

    pub fn get_or_create_ata(
        &mut self,
        sender: AppUser,
        owner: &Pubkey,
        mint: &Pubkey,
    ) -> TestResult<Pubkey> {
        let ata_address = Self::get_ata(owner, mint);

        // check if the ATA already exists
        match self.litesvm.get_account(&ata_address) {
            // ATA exists, return its address
            Some(_) => Ok(ata_address),
            // ATA doesn't exist, create it
            None => Self::create_ata(&mut self.litesvm, &sender.keypair(), owner, mint),
        }
    }

    pub fn get_ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
        get_associated_token_address(owner, mint)
    }

    pub fn create_ata(
        litesvm: &mut LiteSVM,
        sender: &Keypair,
        owner: &Pubkey,
        mint: &Pubkey,
    ) -> TestResult<Pubkey> {
        create_associated_token_account(litesvm, owner, mint, sender)
            .map_err(TestError::from_unknown)
    }

    pub fn create_account(
        &mut self,
        sender: AppUser,
        new_account: Option<Keypair>,
        space: usize,
        owner: &Pubkey,
    ) -> TestResult<(TransactionMetadata, Keypair)> {
        let account_keypair = new_account.unwrap_or(Keypair::new());
        let signers = &[&sender.keypair(), &account_keypair];

        let lamports = self
            .litesvm
            .get_sysvar::<solana_program::sysvar::rent::Rent>()
            .minimum_balance(space);

        let ix = solana_system_interface::instruction::create_account(
            &sender.pubkey().to_bytes().into(),
            &account_keypair.pubkey().to_bytes().into(),
            lamports,
            space as u64,
            &owner.to_bytes().into(),
        );

        let ix_legacy = solana_instruction::Instruction {
            program_id: addr_to_sol_pubkey(&ix.program_id),
            accounts: ix
                .accounts
                .into_iter()
                .map(|x| solana_instruction::AccountMeta {
                    pubkey: addr_to_sol_pubkey(&x.pubkey),
                    is_signer: x.is_signer,
                    is_writable: x.is_writable,
                })
                .collect(),
            data: ix.data,
        };

        let tx_metadata = extension::send_tx(
            &mut self.litesvm,
            &[ix_legacy],
            signers,
            self.is_log_displayed,
        )?;

        Ok((tx_metadata, account_keypair))
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(false)
    }
}

pub fn get_test_error_from_logs(logs: &[String]) -> TestError {
    match TestError::parse_program_error(logs) {
        Some(program_error) => TestError::from_unknown(program_error),
        None => {
            TestError::from_unknown("Not a ProgramError and custom_program_error_idx isn't found")
        }
    }
}

pub fn get_program_size(program_name: &str) -> TestResult<u64> {
    let program_path = &get_program_path(program_name);

    std::fs::metadata(program_path)
        .map(|x| x.len())
        .map_err(|_| TestError {
            info: format!("{} program isn't found!", program_path),
            index: None,
        })
}

fn get_program_path(program_name: &str) -> String {
    const PROGRAM_PATH: &str = "../../target/deploy/";
    format!("{}{}.so", PROGRAM_PATH, program_name)
}

fn upload_program(litesvm: &mut LiteSVM, program_name: &str, program_id: &Pubkey) {
    deploy_program(litesvm, program_id, &get_program_path(program_name)).unwrap()
}

pub mod extension {
    use super::*;

    pub fn get_account_data(app: &App, pubkey: &pinocchio::pubkey::Pubkey) -> TestResult<Vec<u8>> {
        app.litesvm
            .get_account(&pin_to_sol_pubkey(pubkey))
            .map(|x| x.data)
            .ok_or(TestError::from_raw_error("The account isn't found"))
    }

    pub fn send_tx<S>(
        litesvm: &mut LiteSVM,
        instructions: &[Instruction],
        signers: &S,
        is_log_displayed: bool,
    ) -> TestResult<TransactionMetadata>
    where
        S: Signers + ?Sized,
    {
        // to avoid AlreadyProcessed error
        litesvm.expire_blockhash();

        let transaction = Transaction::new_signed_with_payer(
            instructions,
            signers.pubkeys().first(),
            signers,
            litesvm.latest_blockhash(),
        );

        match litesvm.send_transaction(transaction) {
            Ok(x) => {
                let logs = &x.logs;

                if is_log_displayed {
                    println!("Transaction logs: {:#?}\n", logs);
                }

                Ok(x)
            }
            Err(e) => {
                let logs = &e.meta.logs;

                if is_log_displayed {
                    println!("Transaction logs: {:#?}\n", logs);
                }

                Err(get_test_error_from_logs(logs))
            }
        }
    }

    pub fn send_tx_with_ix<S>(
        app: &mut App,
        program_id: &Pubkey,
        accounts: &[AccountMeta],
        instruction_data: &[u8],
        signers: &S,
        remaining_accounts: &[AccountMeta],
    ) -> TestResult<TransactionMetadata>
    where
        S: Signers + ?Sized,
    {
        let ix = Instruction {
            program_id: *program_id,
            accounts: [accounts, remaining_accounts].concat(),
            data: instruction_data.to_vec(),
        };

        send_tx(&mut app.litesvm, &[ix], signers, app.is_log_displayed)
    }
}

pub fn assert_error<T: Sized + std::fmt::Debug>(err: TestError, expected: T) {
    let expected_error_name = format!("{:?}", expected).replace("\"", "");

    let error = err.info;
    let contains_name = error.contains(&expected_error_name);

    pretty_assertions::assert_eq!(
        "",
        if contains_name { "" } else { " " },
        "\n\n✅ Expected error:\n{}\n\n❌ Received error:\n{}",
        expected_error_name,
        error
    );
}
