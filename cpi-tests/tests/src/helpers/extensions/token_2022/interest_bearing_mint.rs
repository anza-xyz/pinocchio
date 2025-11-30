use {
    crate::helpers::suite::{
        core::{
            extension::{get_account_data, send_tx},
            App, ProgramId,
        },
        types::{
            addr_to_sol_pubkey, pin_pubkey_to_addr, to_optional_non_zero_pubkey, AppUser,
            SolPubkey, Target, TestError, TestResult,
        },
    },
    litesvm::types::TransactionMetadata,
    pinocchio::pubkey::Pubkey,
    spl_token_2022_interface::{
        extension::{interest_bearing_mint::InterestBearingConfig, BaseStateWithExtensions, StateWithExtensions},
        state::Mint,
    },
};

pub trait Token2022InterestBearingMintExtension {
    fn token_2022_try_initialize_interest_bearing_mint(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        rate_authority: Option<&Pubkey>,
        rate: i16,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_update_interest_bearing_mint_rate(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        authority: &Pubkey,
        rate: i16,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_update_interest_bearing_mint_rate_multisig(
        &mut self,
        target: Target,
        mint: &Pubkey,
        multisig_authority: &Pubkey,
        signers: &[AppUser],
        rate: i16,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_query_interest_bearing_mint(
        &self,
        target: Target,
        mint: &Pubkey,
    ) -> TestResult<InterestBearingConfig>;
}

impl Token2022InterestBearingMintExtension for App {
    fn token_2022_try_initialize_interest_bearing_mint(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        rate_authority: Option<&Pubkey>,
        rate: i16,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&sender.keypair()];

        let ix = spl_token_2022_interface::extension::interest_bearing_mint::instruction::initialize(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(mint),
            rate_authority.map(pin_pubkey_to_addr),
            rate,
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

    fn token_2022_try_update_interest_bearing_mint_rate(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        authority: &Pubkey,
        rate: i16,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&sender.keypair()];
        let authority_signers = &[&pin_pubkey_to_addr(&sender.pubkey().to_bytes())];

        let ix = spl_token_2022_interface::extension::interest_bearing_mint::instruction::update_rate(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(mint),
            &pin_pubkey_to_addr(authority),
            authority_signers,
            rate,
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

    fn token_2022_try_update_interest_bearing_mint_rate_multisig(
        &mut self,
        target: Target,
        mint: &Pubkey,
        multisig_authority: &Pubkey,
        signers: &[AppUser],
        rate: i16,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signer_keypairs: Vec<_> = signers.iter().map(|s| s.keypair()).collect();

        // create authority signers for the instruction
        let authority_signers: Vec<_> = signers
            .iter()
            .map(|s| pin_pubkey_to_addr(&s.pubkey().to_bytes()))
            .collect();
        let authority_signer_refs: Vec<_> = authority_signers.iter().collect();

        let ix = spl_token_2022_interface::extension::interest_bearing_mint::instruction::update_rate(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(mint),
            &pin_pubkey_to_addr(multisig_authority),
            &authority_signer_refs,
            rate,
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
            &signer_keypairs,
            self.is_log_displayed,
        )
    }

    fn token_2022_query_interest_bearing_mint(
        &self,
        target: Target,
        mint: &Pubkey,
    ) -> TestResult<InterestBearingConfig> {
        let data = &get_account_data(self, mint)?;

        match target {
            Target::Spl => {
                // parse the mint account with extensions
                let mint_with_extensions =
                    StateWithExtensions::<Mint>::unpack(data).map_err(TestError::from_raw_error)?;

                // get the extension
                mint_with_extensions
                    .get_extension::<InterestBearingConfig>()
                    .map(|&x| x)
                    .map_err(TestError::from_raw_error)
            }
            Target::Proxy => {
                use pinocchio_token_2022::extension::interest_bearing_mint::state::InterestBearingConfig as PinocchioInterestBearingConfig;

                let state =
                    PinocchioInterestBearingConfig::from_bytes(data).map_err(TestError::from_raw_error)?;

                Ok(InterestBearingConfig {
                    rate_authority: to_optional_non_zero_pubkey(state.rate_authority()),
                    initialization_timestamp: state.initialization_timestamp().into(),
                    last_update_timestamp: state.last_update_timestamp().into(),
                    pre_update_average_rate: state.pre_update_average_rate().into(),
                    current_rate: state.current_rate().into(),
                })
            }
        }
    }
}
