use {
    crate::helpers::suite::{
        core::{extension::send_tx, App, ProgramId},
        types::{addr_to_sol_pubkey, pin_pubkey_to_addr, AppUser, Target, TestError, TestResult},
    },
    litesvm::types::TransactionMetadata,
    pinocchio::pubkey::Pubkey,
    solana_keypair::Keypair,
    solana_program_pack::Pack,
    spl_token_2022_interface::{instruction::MAX_SIGNERS, state::Multisig},
};

pub trait Token2022InitializeMultisigExtension {
    fn token_2022_try_create_multisig(
        &mut self,
        sender: AppUser,
        mint: Option<Keypair>,
    ) -> TestResult<(TransactionMetadata, Keypair)>;

    fn token_2022_try_initialize_multisig(
        &mut self,
        target: Target,
        sender: AppUser,
        multisig: &Pubkey,
        required_signers: u8,
        signer_pubkeys: &[Pubkey],
    ) -> TestResult<TransactionMetadata>;
}

impl Token2022InitializeMultisigExtension for App {
    fn token_2022_try_create_multisig(
        &mut self,
        sender: AppUser,
        mint: Option<Keypair>,
    ) -> TestResult<(TransactionMetadata, Keypair)> {
        let ProgramId {
            token_2022_program, ..
        } = self.program_id;

        self.create_account(
            sender,
            mint,
            Multisig::get_packed_len(),
            &token_2022_program,
        )
    }

    fn token_2022_try_initialize_multisig(
        &mut self,
        target: Target,
        sender: AppUser,
        multisig: &Pubkey,
        required_signers: u8,
        signer_pubkeys: &[Pubkey],
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&sender.keypair()];

        let signer_pubkeys: &Vec<_> = &signer_pubkeys.iter().map(pin_pubkey_to_addr).collect();
        let default_address = &solana_address::Address::default();
        let signer_pubkeys_buffer: [&solana_address::Address; MAX_SIGNERS] =
            core::array::from_fn(|i| signer_pubkeys.get(i).unwrap_or(default_address));

        let ix = spl_token_2022_interface::instruction::initialize_multisig(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(multisig),
            &signer_pubkeys_buffer,
            required_signers,
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
}
