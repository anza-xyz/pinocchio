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
        extension::{cpi_guard::CpiGuard, BaseStateWithExtensions, StateWithExtensions},
        state::Account,
    },
};

pub trait Token2022CpiGuardExtension {
    fn token_2022_try_enable_cpi_guard(
        &mut self,
        target: Target,
        sender: AppUser,
        token_account: &Pubkey,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_enable_cpi_guard_mutisig(
        &mut self,
        target: Target,
        token_account: &Pubkey,
        multisig_authority: &Pubkey,
        signers: &[AppUser],
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_disable_cpi_guard(
        &mut self,
        target: Target,
        sender: AppUser,
        token_account: &Pubkey,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_disable_cpi_guard_mutisig(
        &mut self,
        target: Target,
        token_account: &Pubkey,
        multisig_authority: &Pubkey,
        signers: &[AppUser],
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_query_cpi_guard(
        &self,
        target: Target,
        token_account: &Pubkey,
    ) -> TestResult<CpiGuard>;
}

impl Token2022CpiGuardExtension for App {
    fn token_2022_try_enable_cpi_guard(
        &mut self,
        target: Target,
        sender: AppUser,
        token_account: &Pubkey,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&sender.keypair()];

        let signer_pubkeys: &[&solana_address::Address] =
            &[&pin_pubkey_to_addr(&sender.pubkey().to_bytes())];

        let ix = spl_token_2022_interface::extension::cpi_guard::instruction::enable_cpi_guard(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(token_account),
            &sender.pubkey().to_bytes().into(),
            &signer_pubkeys,
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
            ix_legacy.accounts.extend(additional_accounts);
        }

        send_tx(
            &mut self.litesvm,
            &[ix_legacy],
            signers,
            self.is_log_displayed,
        )
    }

    fn token_2022_try_enable_cpi_guard_mutisig(
        &mut self,
        target: Target,
        token_account: &Pubkey,
        multisig_authority: &Pubkey,
        signers: &[AppUser],
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signer_keypairs: Vec<_> = signers.iter().map(|s| s.keypair()).collect();
        let signer_pubkeys: Vec<_> = signers
            .iter()
            .map(|s| pin_pubkey_to_addr(&s.pubkey().to_bytes()))
            .collect();
        let signer_pubkey_refs: Vec<_> = signer_pubkeys.iter().collect();

        let ix = spl_token_2022_interface::extension::cpi_guard::instruction::enable_cpi_guard(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(token_account),
            &pin_pubkey_to_addr(multisig_authority),
            &signer_pubkey_refs,
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
            ix_legacy.accounts.extend(additional_accounts);
        }

        send_tx(
            &mut self.litesvm,
            &[ix_legacy],
            &signer_keypairs,
            self.is_log_displayed,
        )
    }

    fn token_2022_try_disable_cpi_guard(
        &mut self,
        target: Target,
        sender: AppUser,
        token_account: &Pubkey,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&sender.keypair()];

        let signer_pubkeys: &[&solana_address::Address] =
            &[&pin_pubkey_to_addr(&sender.pubkey().to_bytes())];

        let ix = spl_token_2022_interface::extension::cpi_guard::instruction::disable_cpi_guard(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(token_account),
            &sender.pubkey().to_bytes().into(),
            &signer_pubkeys,
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
            ix_legacy.accounts.extend(additional_accounts);
        }

        send_tx(
            &mut self.litesvm,
            &[ix_legacy],
            signers,
            self.is_log_displayed,
        )
    }

    fn token_2022_try_disable_cpi_guard_mutisig(
        &mut self,
        target: Target,
        token_account: &Pubkey,
        multisig_authority: &Pubkey,
        signers: &[AppUser],
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signer_keypairs: Vec<_> = signers.iter().map(|s| s.keypair()).collect();
        let signer_pubkeys: Vec<_> = signers
            .iter()
            .map(|s| pin_pubkey_to_addr(&s.pubkey().to_bytes()))
            .collect();
        let signer_pubkey_refs: Vec<_> = signer_pubkeys.iter().collect();

        let ix = spl_token_2022_interface::extension::cpi_guard::instruction::disable_cpi_guard(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(token_account),
            &pin_pubkey_to_addr(multisig_authority),
            &signer_pubkey_refs,
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
            ix_legacy.accounts.extend(additional_accounts);
        }

        send_tx(
            &mut self.litesvm,
            &[ix_legacy],
            &signer_keypairs,
            self.is_log_displayed,
        )
    }

    fn token_2022_query_cpi_guard(
        &self,
        target: Target,
        token_account: &Pubkey,
    ) -> TestResult<CpiGuard> {
        let data = &get_account_data(self, token_account)?;

        match target {
            Target::Spl => {
                // parse the token account with extensions
                let account_with_extensions = StateWithExtensions::<Account>::unpack(data)
                    .map_err(TestError::from_raw_error)?;

                // get the CpiGuard extension
                account_with_extensions
                    .get_extension::<CpiGuard>()
                    .map(|&x| x)
                    .map_err(TestError::from_raw_error)
            }
            Target::Proxy => {
                use pinocchio_token_2022::extension::cpi_guard::state::CpiGuard as PinocchioCpiGuard;

                let state =
                    PinocchioCpiGuard::from_bytes(data).map_err(TestError::from_raw_error)?;

                Ok(CpiGuard {
                    lock_cpi: state.lock_cpi().into(),
                })
            }
        }
    }
}
