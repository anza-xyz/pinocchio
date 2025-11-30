use {
    crate::helpers::suite::{
        core::{
            extension::{get_account_data, send_tx},
            App, ProgramId,
        },
        types::{
            addr_to_sol_pubkey, pin_pubkey_to_addr, AppUser, SolPubkey, Target, TestError,
            TestResult,
        },
    },
    litesvm::types::TransactionMetadata,
    pinocchio::pubkey::Pubkey,
    spl_token_2022_interface::{
        extension::{default_account_state::DefaultAccountState, BaseStateWithExtensions, StateWithExtensions},
        state::{AccountState, Mint},
    },
};

pub trait Token2022DefaultAccountStateExtension {
    fn token_2022_try_initialize_default_account_state(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        state: AccountState,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_update_default_account_state(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        freeze_authority: &Pubkey,
        state: AccountState,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_update_default_account_state_multisig(
        &mut self,
        target: Target,
        mint: &Pubkey,
        multisig_freeze_authority: &Pubkey,
        signers: &[AppUser],
        state: AccountState,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_query_default_account_state(
        &self,
        target: Target,
        mint: &Pubkey,
    ) -> TestResult<DefaultAccountState>;
}

impl Token2022DefaultAccountStateExtension for App {
    fn token_2022_try_initialize_default_account_state(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        state: AccountState,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&sender.keypair()];

        let ix = spl_token_2022_interface::extension::default_account_state::instruction::initialize_default_account_state(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(mint),
            &state,
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

    fn token_2022_try_update_default_account_state(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        freeze_authority: &Pubkey,
        state: AccountState,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&sender.keypair()];
        let authority_signers = &[&pin_pubkey_to_addr(&SolPubkey::pubkey(&sender).to_bytes())];

        let ix = spl_token_2022_interface::extension::default_account_state::instruction::update_default_account_state(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(mint),
            &pin_pubkey_to_addr(freeze_authority),
            authority_signers,
            &state,
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

    fn token_2022_try_update_default_account_state_multisig(
        &mut self,
        target: Target,
        mint: &Pubkey,
        multisig_freeze_authority: &Pubkey,
        signers: &[AppUser],
        state: AccountState,
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
            .map(|s| pin_pubkey_to_addr(&SolPubkey::pubkey(s).to_bytes()))
            .collect();
        let authority_signer_refs: Vec<_> = authority_signers.iter().collect();

        let ix = spl_token_2022_interface::extension::default_account_state::instruction::update_default_account_state(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(mint),
            &pin_pubkey_to_addr(multisig_freeze_authority),
            &authority_signer_refs,
            &state,
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

    fn token_2022_query_default_account_state(
        &self,
        target: Target,
        mint: &Pubkey,
    ) -> TestResult<DefaultAccountState> {
        let data = &get_account_data(self, mint)?;

        match target {
            Target::Spl => {
                // parse the mint account with extensions
                let mint_with_extensions =
                    StateWithExtensions::<Mint>::unpack(data).map_err(TestError::from_raw_error)?;

                // get the DefaultAccountState extension
                mint_with_extensions
                    .get_extension::<DefaultAccountState>()
                    .map(|&x| x)
                    .map_err(TestError::from_raw_error)
            }
            Target::Proxy => {
                use pinocchio_token_2022::extension::default_account_state::state::DefaultAccountStateConfig as PinocchioDefaultAccountStateConfig;

                let state =
                    PinocchioDefaultAccountStateConfig::from_bytes(data).map_err(TestError::from_raw_error)?;

                Ok(DefaultAccountState {
                    state: state.state().into(),
                })
            }
        }
    }
}
