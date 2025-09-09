use {
    crate::helpers::suite::{
        core::{extension::send_tx, App, ProgramId},
        types::{
            addr_to_sol_pubkey, pin_pubkey_to_addr, pin_to_sol_pubkey, to_c_option, AppUser,
            SolPubkey, TestError, TestResult,
        },
    },
    litesvm::types::TransactionMetadata,
    pinocchio::pubkey::Pubkey,
    solana_keypair::Keypair,
    solana_program_pack::Pack,
    solana_signer::Signer,
    solana_system_interface::instruction::create_account,
};

pub trait Token2022InitializeMintExtension {
    fn token_2022_try_create_mint_account(
        &mut self,
        sender: AppUser,
        mint: Option<Keypair>,
        extensions: Option<&[spl_token_2022_interface::extension::ExtensionType]>,
    ) -> TestResult<(TransactionMetadata, Keypair)>;

    /// execute token-2022 instruction directly
    fn token_2022_try_initialize_mint(
        &mut self,
        sender: AppUser,
        mint: &Pubkey,
        decimals: u8,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata>;

    /// execute token-2022 instruction using proxy program
    fn token_2022_proxy_try_initialize_mint(
        &mut self,
        sender: AppUser,
        mint: &Pubkey,
        decimals: u8,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata>;

    /// read token-2022 state using spl interface
    fn token_2022_query_mint_state(
        &self,
        mint: &Pubkey,
    ) -> TestResult<spl_token_2022_interface::state::Mint>;

    /// read token-2022 state using pinocchio interface
    fn token_2022_proxy_query_mint_state(
        &self,
        mint: &Pubkey,
    ) -> TestResult<spl_token_2022_interface::state::Mint>;
}

impl Token2022InitializeMintExtension for App {
    fn token_2022_try_create_mint_account(
        &mut self,
        sender: AppUser,
        mint: Option<Keypair>,
        extensions: Option<&[spl_token_2022_interface::extension::ExtensionType]>,
    ) -> TestResult<(TransactionMetadata, Keypair)> {
        let ProgramId {
            token_2022_program, ..
        } = self.program_id;

        let mint_keypair = mint.unwrap_or(Keypair::new());
        let signers = &[&sender.keypair(), &mint_keypair];

        let account_size = match extensions {
            Some(x) => {
                spl_token_2022_interface::extension::ExtensionType::try_calculate_account_len::<
                    spl_token_2022_interface::state::Mint,
                >(x)
                .map_err(|e| {
                    TestError::from_raw_error(format!(
                        "Failed to calculate account length: {:?}",
                        e
                    ))
                })?
            }
            None => spl_token_2022_interface::state::Mint::LEN,
        };

        let lamports = self
            .litesvm
            .get_sysvar::<solana_program::sysvar::rent::Rent>()
            .minimum_balance(account_size);

        let ix = create_account(
            &sender.pubkey().to_bytes().into(),
            &mint_keypair.pubkey().to_bytes().into(),
            lamports,
            account_size as u64,
            &token_2022_program.to_bytes().into(),
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

        let tx_metadata = send_tx(
            &mut self.litesvm,
            &[ix_legacy],
            signers,
            self.is_log_displayed,
        )?;

        Ok((tx_metadata, mint_keypair))
    }

    fn token_2022_try_initialize_mint(
        &mut self,
        sender: AppUser,
        mint: &Pubkey,
        decimals: u8,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata> {
        // programs
        let ProgramId {
            token_2022_program, ..
        } = self.program_id;

        // signers
        let signers = &[sender.keypair()];

        let ix = spl_token_2022_interface::instruction::initialize_mint(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(mint),
            &pin_pubkey_to_addr(mint_authority),
            freeze_authority.map(pin_pubkey_to_addr).as_ref(),
            decimals,
        )
        .map_err(TestError::from_raw_error)?;

        // convert Instruction v3.0.0 to Instruction v2.3.0
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

        send_tx(
            &mut self.litesvm,
            &[ix_legacy],
            signers,
            self.is_log_displayed,
        )
    }

    fn token_2022_proxy_try_initialize_mint(
        &mut self,
        sender: AppUser,
        mint: &Pubkey,
        decimals: u8,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata> {
        // programs
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        // signers
        let signers = &[sender.keypair()];

        // instruction builder checks program_id, token_2022_program must be specified here
        let ix = spl_token_2022_interface::instruction::initialize_mint(
            &pin_pubkey_to_addr(&token_2022_program.to_bytes()),
            &pin_pubkey_to_addr(mint),
            &pin_pubkey_to_addr(mint_authority),
            freeze_authority.map(pin_pubkey_to_addr).as_ref(),
            decimals,
        )
        .map_err(TestError::from_raw_error)?;

        // required by runtime to validate programs
        let additional_accounts = [solana_instruction::AccountMeta::new_readonly(
            token_2022_program,
            false,
        )];

        // convert Instruction v3.0.0 to Instruction v2.3.0
        // program_id should be replaced now: token_2022_program -> token_2022_proxy
        let ix_legacy = solana_instruction::Instruction {
            program_id: token_2022_proxy,
            accounts: ix
                .accounts
                .into_iter()
                .map(|x| solana_instruction::AccountMeta {
                    pubkey: addr_to_sol_pubkey(&x.pubkey),
                    is_signer: x.is_signer,
                    is_writable: x.is_writable,
                })
                .chain(additional_accounts.into_iter())
                .collect(),
            data: ix.data,
        };

        send_tx(
            &mut self.litesvm,
            &[ix_legacy],
            signers,
            self.is_log_displayed,
        )
    }

    fn token_2022_query_mint_state(
        &self,
        mint: &Pubkey,
    ) -> TestResult<spl_token_2022_interface::state::Mint> {
        self.litesvm
            .get_account(&pin_to_sol_pubkey(mint))
            .map(|x| spl_token_2022_interface::state::Mint::unpack_from_slice(&x.data))
            .transpose()
            .map_err(TestError::from_raw_error)?
            .ok_or(TestError::from_raw_error("The state isn't found"))
    }

    fn token_2022_proxy_query_mint_state(
        &self,
        mint: &Pubkey,
    ) -> TestResult<spl_token_2022_interface::state::Mint> {
        let data = &self
            .litesvm
            .get_account(&pin_to_sol_pubkey(mint))
            .map(|x| x.data)
            .ok_or(TestError::from_raw_error("The state isn't found"))?;

        let state = unsafe { pinocchio_token_2022::state::Mint::from_bytes_unchecked(data) };

        Ok(spl_token_2022_interface::state::Mint {
            mint_authority: to_c_option(state.mint_authority().map(pin_pubkey_to_addr)),
            supply: state.supply(),
            decimals: state.decimals(),
            is_initialized: state.is_initialized(),
            freeze_authority: to_c_option(state.freeze_authority().map(pin_pubkey_to_addr)),
        })
    }
}
