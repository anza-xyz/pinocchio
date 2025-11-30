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
        extension::{
            metadata_pointer::MetadataPointer, BaseStateWithExtensions, StateWithExtensions,
        },
        state::Mint,
    },
};

pub trait Token2022MetadataPointerExtension {
    fn token_2022_try_initialize_metadata_pointer(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        authority: Option<&Pubkey>,
        metadata_address: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_update_metadata_pointer(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        authority: &Pubkey,
        metadata_address: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_update_metadata_pointer_multisig(
        &mut self,
        target: Target,
        mint: &Pubkey,
        multisig_authority: &Pubkey,
        signers: &[AppUser],
        metadata_address: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_query_metadata_pointer(
        &self,
        target: Target,
        mint: &Pubkey,
    ) -> TestResult<MetadataPointer>;
}

impl Token2022MetadataPointerExtension for App {
    fn token_2022_try_initialize_metadata_pointer(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        authority: Option<&Pubkey>,
        metadata_address: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&sender.keypair()];

        let ix = spl_token_2022_interface::extension::metadata_pointer::instruction::initialize(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(mint),
            authority.map(pin_pubkey_to_addr),
            metadata_address.map(pin_pubkey_to_addr),
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

    fn token_2022_try_update_metadata_pointer(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        authority: &Pubkey,
        metadata_address: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&sender.keypair()];
        let authority_signers = &[&pin_pubkey_to_addr(&sender.pubkey().to_bytes())];

        let ix = spl_token_2022_interface::extension::metadata_pointer::instruction::update(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(mint),
            &pin_pubkey_to_addr(authority),
            authority_signers,
            metadata_address.map(pin_pubkey_to_addr),
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

    fn token_2022_try_update_metadata_pointer_multisig(
        &mut self,
        target: Target,
        mint: &Pubkey,
        multisig_authority: &Pubkey,
        signers: &[AppUser],
        metadata_address: Option<&Pubkey>,
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

        let ix = spl_token_2022_interface::extension::metadata_pointer::instruction::update(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(mint),
            &pin_pubkey_to_addr(multisig_authority),
            &authority_signer_refs,
            metadata_address.map(pin_pubkey_to_addr),
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

    fn token_2022_query_metadata_pointer(
        &self,
        target: Target,
        mint: &Pubkey,
    ) -> TestResult<MetadataPointer> {
        let data = &get_account_data(self, mint)?;

        match target {
            Target::Spl => {
                // parse the mint account with extensions
                let mint_with_extensions =
                    StateWithExtensions::<Mint>::unpack(data).map_err(TestError::from_raw_error)?;

                // get the extension
                mint_with_extensions
                    .get_extension::<MetadataPointer>()
                    .map(|&x| x)
                    .map_err(TestError::from_raw_error)
            }
            Target::Proxy => {
                use pinocchio_token_2022::extension::metadata_pointer::state::MetadataPointer as PinocchioMetadataPointer;

                let state = PinocchioMetadataPointer::from_bytes(data)
                    .map_err(TestError::from_raw_error)?;

                Ok(MetadataPointer {
                    authority: to_optional_non_zero_pubkey(state.authority()),
                    metadata_address: to_optional_non_zero_pubkey(state.metadata_address()),
                })
            }
        }
    }
}
