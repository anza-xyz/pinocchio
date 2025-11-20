use {
    crate::helpers::suite::{
        core::{extension::send_tx, App, ProgramId},
        types::{
            addr_to_sol_pubkey, pin_pubkey_to_addr, AppUser, SolPubkey, TestError, TestResult,
        },
    },
    litesvm::types::TransactionMetadata,
    pinocchio::pubkey::Pubkey,
    solana_keypair::Keypair,
    solana_signer::Signer,
    spl_token_2022_interface::{extension::ExtensionType, state::Account},
};

pub trait Token2022TokenAccountExtension {
    fn token_2022_try_create_and_init_token_account(
        &mut self,
        sender: AppUser,
        owner: &Pubkey,
        mint: &Pubkey,
        extensions: &[ExtensionType],
    ) -> TestResult<(TransactionMetadata, Keypair)>;
}

impl Token2022TokenAccountExtension for App {
    fn token_2022_try_create_and_init_token_account(
        &mut self,
        sender: AppUser,
        owner: &Pubkey,
        mint: &Pubkey,
        extensions: &[ExtensionType],
    ) -> TestResult<(TransactionMetadata, Keypair)> {
        let ProgramId {
            token_2022_program, ..
        } = self.program_id;

        // Calculate space needed for token account with extensions
        let space = ExtensionType::try_calculate_account_len::<Account>(extensions)
            .map_err(TestError::from_raw_error)?;

        // Create the token account keypair
        let token_account_keypair = Keypair::new();
        let signers = &[&sender.keypair(), &token_account_keypair];

        let lamports = self
            .litesvm
            .get_sysvar::<solana_program::sysvar::rent::Rent>()
            .minimum_balance(space);

        let mut instructions = vec![];

        // Create account instruction
        let create_ix = solana_system_interface::instruction::create_account(
            &sender.pubkey().to_bytes().into(),
            &token_account_keypair.pubkey().to_bytes().into(),
            lamports,
            space as u64,
            &token_2022_program.to_bytes().into(),
        );

        instructions.push(solana_instruction::Instruction {
            program_id: addr_to_sol_pubkey(&create_ix.program_id),
            accounts: create_ix
                .accounts
                .into_iter()
                .map(|x| solana_instruction::AccountMeta {
                    pubkey: addr_to_sol_pubkey(&x.pubkey),
                    is_signer: x.is_signer,
                    is_writable: x.is_writable,
                })
                .collect(),
            data: create_ix.data,
        });

        // Initialize immutable owner if needed
        if extensions.contains(&ExtensionType::ImmutableOwner) {
            let init_immutable_ix =
                spl_token_2022_interface::instruction::initialize_immutable_owner(
                    &token_2022_program.to_bytes().into(),
                    &token_account_keypair.pubkey().to_bytes().into(),
                )
                .map_err(TestError::from_raw_error)?;

            instructions.push(solana_instruction::Instruction {
                program_id: addr_to_sol_pubkey(&init_immutable_ix.program_id),
                accounts: init_immutable_ix
                    .accounts
                    .into_iter()
                    .map(|x| solana_instruction::AccountMeta {
                        pubkey: addr_to_sol_pubkey(&x.pubkey),
                        is_signer: x.is_signer,
                        is_writable: x.is_writable,
                    })
                    .collect(),
                data: init_immutable_ix.data,
            });
        }

        // Initialize token account
        let init_account_ix = spl_token_2022_interface::instruction::initialize_account(
            &token_2022_program.to_bytes().into(),
            &token_account_keypair.pubkey().to_bytes().into(),
            &pin_pubkey_to_addr(mint),
            &pin_pubkey_to_addr(owner),
        )
        .map_err(TestError::from_raw_error)?;

        instructions.push(solana_instruction::Instruction {
            program_id: addr_to_sol_pubkey(&init_account_ix.program_id),
            accounts: init_account_ix
                .accounts
                .into_iter()
                .map(|x| solana_instruction::AccountMeta {
                    pubkey: addr_to_sol_pubkey(&x.pubkey),
                    is_signer: x.is_signer,
                    is_writable: x.is_writable,
                })
                .collect(),
            data: init_account_ix.data,
        });

        let tx_metadata = send_tx(
            &mut self.litesvm,
            &instructions,
            signers,
            self.is_log_displayed,
        )?;

        Ok((tx_metadata, token_account_keypair))
    }
}
