use {
    crate::helpers::suite::{
        core::{
            extension::{get_account_data, send_tx},
            App, ProgramId,
        },
        types::{
            addr_to_sol_pubkey, pin_pubkey_to_addr, to_c_option, AppUser, Target, TestError,
            TestResult,
        },
    },
    litesvm::types::TransactionMetadata,
    pinocchio::pubkey::Pubkey,
    pinocchio_token_2022::state::{
        AccountState as PinocchioAccountState, TokenAccount as PinocchioTokenAccount,
    },
    solana_keypair::Keypair,
    solana_program_pack::Pack,
    spl_token_2022_interface::{
        extension::ExtensionType,
        state::{Account, AccountState as SplAccountState},
    },
};

pub trait Token2022InitializeAccountExtension {
    fn token_2022_try_create_token_account(
        &mut self,
        sender: AppUser,
        account: Option<Keypair>,
        extensions: Option<&[ExtensionType]>,
    ) -> TestResult<(TransactionMetadata, Keypair)>;

    fn token_2022_try_initialize_token_account(
        &mut self,
        target: Target,
        sender: AppUser,
        account: &Pubkey,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_query_token_account(
        &self,
        target: Target,
        account: &Pubkey,
    ) -> TestResult<Account>;

    fn token2022_try_create_and_try_initialize_mint(
        &mut self,
        target: Target,
    ) -> TestResult<(TransactionMetadata, Pubkey)>;
}

impl Token2022InitializeAccountExtension for App {
    fn token_2022_try_create_token_account(
        &mut self,
        sender: AppUser,
        account: Option<Keypair>,
        extensions: Option<&[ExtensionType]>,
    ) -> TestResult<(TransactionMetadata, Keypair)> {
        let ProgramId {
            token_2022_program, ..
        } = self.program_id;

        let account_size = match extensions {
            Some(extention_type) => {
                ExtensionType::try_calculate_account_len::<Account>(extention_type)
                    .map_err(TestError::from_raw_error)?
            }
            None => Account::LEN,
        };

        self.create_account(sender, account, account_size, &token_2022_program)
    }

    fn token_2022_try_initialize_token_account(
        &mut self,
        target: Target,
        sender: AppUser,
        account: &Pubkey,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[sender.keypair()];

        let ix = spl_token_2022_interface::instruction::initialize_account(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(account),
            &pin_pubkey_to_addr(mint),
            &pin_pubkey_to_addr(owner),
        )
        .map_err(TestError::from_raw_error)?;

        let additional_accounts = [solana_instruction::AccountMeta::new_readonly(
            token_2022_program,
            false,
        )];

        let mut ix_legacy = solana_instruction::Instruction {
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

        if let Target::Proxy = target {
            ix_legacy.program_id = token_2022_proxy;
            ix_legacy.accounts.extend_from_slice(&additional_accounts);
        }

        send_tx(
            &mut self.litesvm,
            &[ix_legacy],
            signers,
            self.is_log_displayed,
        )
    }

    fn token_2022_query_token_account(
        &self,
        target: Target,
        account: &Pubkey,
    ) -> TestResult<Account> {
        let data = &get_account_data(self, account)?;

        match target {
            Target::Spl => Account::unpack_from_slice(data).map_err(TestError::from_raw_error),
            Target::Proxy => {
                let state = unsafe { PinocchioTokenAccount::from_bytes_unchecked(data) };

                Ok(Account {
                    mint: pin_pubkey_to_addr(&state.mint()),
                    owner: pin_pubkey_to_addr(&state.owner()),
                    amount: state.amount(),
                    delegate: to_c_option(state.delegate().map(pin_pubkey_to_addr)),

                    state: match state.state() {
                        PinocchioAccountState::Uninitialized => SplAccountState::Uninitialized,
                        PinocchioAccountState::Initialized => SplAccountState::Initialized,
                        PinocchioAccountState::Frozen => SplAccountState::Frozen,
                    },
                    is_native: to_c_option(Some(state.is_native() as u64)), // dev :: Some(0) -> False ; Some(1)
                    delegated_amount: state.delegated_amount(),
                    close_authority: to_c_option(state.close_authority().map(pin_pubkey_to_addr)),
                })
            }
        }
    }

    /// dev: quickly create and initialize a mint; returns (TransactionMetadata, mint pubkey)
    fn token2022_try_create_and_try_initialize_mint(
        &mut self,
        target: Target,
    ) -> TestResult<(TransactionMetadata, Pubkey)> {
        use {
            crate::helpers::{
                extensions::token_2022::initialize_mint::Token2022InitializeMintExtension,
                suite::types::SolPubkey,
            },
            solana_signer::Signer,
        };

        let (_, mint_keypair) =
            self.token_2022_try_create_mint_account(AppUser::Admin, None, None)?;

        let mint = &mint_keypair.pubkey().to_bytes();
        let decimals: u8 = 6;
        let mint_authority = AppUser::Admin.pubkey().to_bytes();
        let freeze_authority = Some(AppUser::Bob.pubkey().to_bytes());
        Ok((
            self.token_2022_try_initialize_mint(
                target,
                AppUser::Admin,
                mint,
                decimals,
                &mint_authority,
                freeze_authority.as_ref(),
            )
            .unwrap(),
            mint.clone(),
        ))
    }
}
