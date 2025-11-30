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
    solana_keypair::Keypair,
    solana_program_pack::Pack,
    spl_token_2022_interface::{extension::ExtensionType, state::Mint},
};

pub trait Token2022InitializeMintExtension {
    fn token_2022_try_create_mint_account(
        &mut self,
        sender: AppUser,
        mint: Option<Keypair>,
        extensions: Option<&[ExtensionType]>,
    ) -> TestResult<(TransactionMetadata, Keypair)>;

    fn token_2022_try_initialize_mint(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        decimals: u8,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_query_mint(&self, target: Target, mint: &Pubkey) -> TestResult<Mint>;
}

impl Token2022InitializeMintExtension for App {
    fn token_2022_try_create_mint_account(
        &mut self,
        sender: AppUser,
        mint: Option<Keypair>,
        extensions: Option<&[ExtensionType]>,
    ) -> TestResult<(TransactionMetadata, Keypair)> {
        let ProgramId {
            token_2022_program, ..
        } = self.program_id;

        let account_size = match extensions {
            Some(x) => ExtensionType::try_calculate_account_len::<Mint>(x)
                .map_err(TestError::from_raw_error)?,
            None => Mint::LEN,
        };

        self.create_account(sender, mint, account_size, &token_2022_program)
    }

    fn token_2022_try_initialize_mint(
        &mut self,
        target: Target,
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
            &token_2022_program.to_bytes().into(),
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

    fn token_2022_query_mint(&self, target: Target, mint: &Pubkey) -> TestResult<Mint> {
        let data = &get_account_data(self, mint)?;

        match target {
            Target::Spl => {
                // parse the mint account
                Mint::unpack_from_slice(data).map_err(TestError::from_raw_error)
            }
            Target::Proxy => {
                use pinocchio_token_2022::state::Mint as PinocchioMint;

                let state = unsafe { PinocchioMint::from_bytes_unchecked(data) };

                Ok(Mint {
                    mint_authority: to_c_option(state.mint_authority().map(pin_pubkey_to_addr)),
                    supply: state.supply(),
                    decimals: state.decimals(),
                    is_initialized: state.is_initialized(),
                    freeze_authority: to_c_option(state.freeze_authority().map(pin_pubkey_to_addr)),
                })
            }
        }
    }
}
