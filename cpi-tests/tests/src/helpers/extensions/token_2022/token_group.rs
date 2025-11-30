use {
    crate::helpers::suite::{
        core::{
            extension::{get_account_data, send_tx},
            App, ProgramId,
        },
        types::{
            addr_to_sol_pubkey, pin_pubkey_to_addr, pin_to_sol_pubkey, to_optional_non_zero_pubkey,
            AppUser, SolPubkey, Target, TestError, TestResult,
        },
    },
    litesvm::types::TransactionMetadata,
    pinocchio::pubkey::Pubkey,
    solana_keypair::Keypair,
    solana_signer::Signer,
    spl_pod::bytemuck::pod_from_bytes,
    spl_token_2022_interface::{
        extension::{BaseStateWithExtensions, StateWithExtensions},
        state::Mint,
    },
    spl_token_group_interface::state::{TokenGroup, TokenGroupMember},
};

pub trait Token2022TokenGroupExtension {
    fn token_2022_try_initialize_token_group(
        &mut self,
        target: Target,
        sender: AppUser,
        group: &Pubkey,
        mint: &Pubkey,
        mint_authority: AppUser,
        update_authority: Option<&Pubkey>,
        max_size: u64,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_update_group_max_size(
        &mut self,
        target: Target,
        sender: AppUser,
        group: &Pubkey,
        mint_authority: AppUser,
        update_authority: &Pubkey,
        max_size: u64,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_update_group_authority(
        &mut self,
        target: Target,
        sender: AppUser,
        group: &Pubkey,
        current_authority: &Pubkey,
        new_authority: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_initialize_member(
        &mut self,
        target: Target,
        sender: AppUser,
        group: &Pubkey,
        group_update_authority: &Keypair,
        member: &Pubkey,
        member_mint: &Pubkey,
        member_mint_authority: &Keypair,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_query_token_group(
        &self,
        target: Target,
        group: &Pubkey,
    ) -> TestResult<TokenGroup>;

    fn token_2022_query_token_group_member(
        &self,
        target: Target,
        mint: &Pubkey,
    ) -> TestResult<TokenGroupMember>;
}

impl Token2022TokenGroupExtension for App {
    fn token_2022_try_initialize_token_group(
        &mut self,
        target: Target,
        sender: AppUser,
        group: &Pubkey,
        mint: &Pubkey,
        mint_authority: AppUser,
        update_authority: Option<&Pubkey>,
        max_size: u64,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&sender.keypair(), &mint_authority.keypair()];

        let lamports = self
            .litesvm
            .get_sysvar::<solana_program::sysvar::rent::Rent>()
            .minimum_balance(max_size as usize);
        self.transfer_sol(sender, &pin_to_sol_pubkey(&mint), lamports)?;

        let ix = spl_token_group_interface::instruction::initialize_group(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(group),
            &pin_pubkey_to_addr(mint),
            &mint_authority.pubkey().to_bytes().into(),
            update_authority.map(pin_pubkey_to_addr),
            max_size,
        );

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

    fn token_2022_try_update_group_max_size(
        &mut self,
        target: Target,
        sender: AppUser,
        group: &Pubkey,
        mint_authority: AppUser,
        update_authority: &Pubkey,
        max_size: u64,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&sender.keypair(), &mint_authority.keypair()];

        let ix = spl_token_group_interface::instruction::update_group_max_size(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(group),
            &pin_pubkey_to_addr(update_authority),
            max_size,
        );

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

    fn token_2022_try_update_group_authority(
        &mut self,
        target: Target,
        sender: AppUser,
        group: &Pubkey,
        current_authority: &Pubkey,
        new_authority: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&sender.keypair()];

        let ix = spl_token_group_interface::instruction::update_group_authority(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(group),
            &pin_pubkey_to_addr(current_authority),
            new_authority.map(pin_pubkey_to_addr),
        );

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

    fn token_2022_try_initialize_member(
        &mut self,
        target: Target,
        sender: AppUser,
        group: &Pubkey,
        group_update_authority: &Keypair,
        member: &Pubkey,
        member_mint: &Pubkey,
        member_mint_authority: &Keypair,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[
            &sender.keypair(),
            member_mint_authority,
            group_update_authority,
        ];

        let lamports = self
            .litesvm
            .get_sysvar::<solana_program::sysvar::rent::Rent>()
            .minimum_balance(core::mem::size_of::<TokenGroupMember>());
        self.transfer_sol(sender, &pin_to_sol_pubkey(&member), lamports)?;

        let ix = spl_token_group_interface::instruction::initialize_member(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(member),
            &pin_pubkey_to_addr(member_mint),
            &member_mint_authority.pubkey().to_bytes().into(),
            &pin_pubkey_to_addr(group),
            &group_update_authority.pubkey().to_bytes().into(),
        );

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

    fn token_2022_query_token_group(
        &self,
        target: Target,
        group: &Pubkey,
    ) -> TestResult<TokenGroup> {
        let data = &get_account_data(self, group)?;

        match target {
            Target::Spl => {
                // Parse the mint account with extensions
                let mint_with_extensions =
                    StateWithExtensions::<Mint>::unpack(data).map_err(TestError::from_raw_error)?;

                // Get extension bytes
                let extension_bytes = mint_with_extensions
                    .get_extension_bytes::<TokenGroup>()
                    .map_err(TestError::from_raw_error)?;

                // Deserialize extension bytes
                pod_from_bytes::<TokenGroup>(extension_bytes)
                    .map(|&x| x)
                    .map_err(TestError::from_raw_error)
            }
            Target::Proxy => {
                use pinocchio_token_2022::extension::token_group::state::TokenGroup as PinocchioTokenGroup;

                let state =
                    PinocchioTokenGroup::from_bytes(data).map_err(TestError::from_raw_error)?;

                Ok(TokenGroup {
                    update_authority: to_optional_non_zero_pubkey(state.update_authority()),
                    mint: pin_pubkey_to_addr(state.mint()),
                    size: state.size().into(),
                    max_size: state.max_size().into(),
                })
            }
        }
    }

    fn token_2022_query_token_group_member(
        &self,
        target: Target,
        mint: &Pubkey,
    ) -> TestResult<TokenGroupMember> {
        let data = &get_account_data(self, mint)?;

        match target {
            Target::Spl => {
                // Parse the mint account with extensions
                let mint_with_extensions =
                    StateWithExtensions::<Mint>::unpack(data).map_err(TestError::from_raw_error)?;

                // Get extension bytes
                let extension_bytes = mint_with_extensions
                    .get_extension_bytes::<TokenGroupMember>()
                    .map_err(TestError::from_raw_error)?;

                // Deserialize extension bytes
                pod_from_bytes::<TokenGroupMember>(extension_bytes)
                    .map(|&x| x)
                    .map_err(TestError::from_raw_error)
            }
            Target::Proxy => {
                use pinocchio_token_2022::extension::token_group::state::TokenGroupMember as PinocchioTokenGroupMember;

                let state = PinocchioTokenGroupMember::from_bytes(data)
                    .map_err(TestError::from_raw_error)?;

                Ok(TokenGroupMember {
                    mint: pin_pubkey_to_addr(state.mint()),
                    group: pin_pubkey_to_addr(state.group()),
                    member_number: state.member_number().into(),
                })
            }
        }
    }
}
