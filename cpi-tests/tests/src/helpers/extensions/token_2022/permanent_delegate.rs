use {
    crate::helpers::suite::{
        core::{
            extension::{get_account_data, send_tx},
            App, ProgramId,
        },
        types::{
            addr_to_sol_pubkey, pin_pubkey_to_addr, to_optional_non_zero_pubkey, AppUser, Target,
            TestError, TestResult,
        },
    },
    litesvm::types::TransactionMetadata,
    pinocchio::pubkey::Pubkey,
    spl_token_2022_interface::{
        extension::{
            permanent_delegate::PermanentDelegate, BaseStateWithExtensions, StateWithExtensions,
        },
        state::Mint,
    },
};

pub trait Token2022PermanentDelegateExtension {
    fn token_2022_try_initialize_permanent_delegate(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        delegate: &Pubkey,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_query_permanent_delegate(
        &self,
        target: Target,
        mint: &Pubkey,
    ) -> TestResult<PermanentDelegate>;
}

impl Token2022PermanentDelegateExtension for App {
    fn token_2022_try_initialize_permanent_delegate(
        &mut self,
        target: Target,
        sender: AppUser,
        mint: &Pubkey,
        delegate: &Pubkey,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&sender.keypair()];

        let ix = spl_token_2022_interface::instruction::initialize_permanent_delegate(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(mint),
            &pin_pubkey_to_addr(delegate),
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

    fn token_2022_query_permanent_delegate(
        &self,
        target: Target,
        mint: &Pubkey,
    ) -> TestResult<PermanentDelegate> {
        let data = &get_account_data(self, mint)?;

        match target {
            Target::Spl => {
                // parse the mint account with extensions
                let mint_with_extensions =
                    StateWithExtensions::<Mint>::unpack(data).map_err(TestError::from_raw_error)?;

                // get the extension
                mint_with_extensions
                    .get_extension::<PermanentDelegate>()
                    .map(|&x| x)
                    .map_err(TestError::from_raw_error)
            }
            Target::Proxy => {
                use pinocchio_token_2022::extension::permanent_delegate::state::PermanentDelegate as PinocchioPermanentDelegate;

                let state = PinocchioPermanentDelegate::from_bytes(data)
                    .map_err(TestError::from_raw_error)?;

                Ok(PermanentDelegate {
                    delegate: to_optional_non_zero_pubkey(state.delegate()),
                })
            }
        }
    }
}
