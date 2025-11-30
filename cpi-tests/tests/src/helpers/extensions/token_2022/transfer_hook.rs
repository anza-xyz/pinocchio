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
        extension::{transfer_hook::TransferHook, BaseStateWithExtensions, StateWithExtensions},
        state::Mint,
    },
};

pub trait Token2022TransferHookExtension {
    fn token_2022_try_initialize_transfer_hook(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        authority: Option<&Pubkey>,
        transfer_hook_program_id: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_update_transfer_hook(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        authority: &Pubkey,
        transfer_hook_program_id: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_update_transfer_hook_multisig(
        &mut self,
        target: Target,
        mint: &Pubkey,
        multisig_authority: &Pubkey,
        signers: &[AppUser],
        transfer_hook_program_id: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_query_transfer_hook(
        &self,
        target: Target,
        mint: &Pubkey,
    ) -> TestResult<TransferHook>;
}

impl Token2022TransferHookExtension for App {
    fn token_2022_try_initialize_transfer_hook(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        authority: Option<&Pubkey>,
        transfer_hook_program_id: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&sender.keypair()];

        let ix = spl_token_2022_interface::extension::transfer_hook::instruction::initialize(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(mint),
            authority.map(|a| pin_pubkey_to_addr(a)),
            transfer_hook_program_id.map(|p| pin_pubkey_to_addr(p)),
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

    fn token_2022_try_update_transfer_hook(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        authority: &Pubkey,
        transfer_hook_program_id: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&sender.keypair()];
        let authority_signers = &[&pin_pubkey_to_addr(&sender.pubkey().to_bytes())];

        let ix = spl_token_2022_interface::extension::transfer_hook::instruction::update(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(mint),
            &pin_pubkey_to_addr(authority),
            authority_signers,
            transfer_hook_program_id.map(|p| pin_pubkey_to_addr(p)),
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

    fn token_2022_try_update_transfer_hook_multisig(
        &mut self,
        target: Target,
        mint: &Pubkey,
        multisig_authority: &Pubkey,
        signers: &[AppUser],
        transfer_hook_program_id: Option<&Pubkey>,
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

        let ix = spl_token_2022_interface::extension::transfer_hook::instruction::update(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(mint),
            &pin_pubkey_to_addr(multisig_authority),
            &authority_signer_refs,
            transfer_hook_program_id.map(|p| pin_pubkey_to_addr(p)),
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

    fn token_2022_query_transfer_hook(
        &self,
        target: Target,
        mint: &Pubkey,
    ) -> TestResult<TransferHook> {
        let data = &get_account_data(self, mint)?;

        match target {
            Target::Spl => {
                // parse the mint account with extensions
                let mint_with_extensions =
                    StateWithExtensions::<Mint>::unpack(data).map_err(TestError::from_raw_error)?;

                // get the TransferHook extension
                mint_with_extensions
                    .get_extension::<TransferHook>()
                    .map(|&x| x)
                    .map_err(TestError::from_raw_error)
            }
            Target::Proxy => {
                use pinocchio_token_2022::extension::transfer_hook::state::TransferHook as PinocchioTransferHook;

                let state =
                    PinocchioTransferHook::from_bytes(data).map_err(TestError::from_raw_error)?;

                Ok(TransferHook {
                    authority: to_optional_non_zero_pubkey(state.authority()),
                    program_id: to_optional_non_zero_pubkey(state.program_id()),
                })
            }
        }
    }
}


