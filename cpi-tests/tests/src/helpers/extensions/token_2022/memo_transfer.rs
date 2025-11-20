use {
    crate::helpers::suite::{
        core::{extension::send_tx, App, ProgramId},
        types::{
            addr_to_sol_pubkey, pin_pubkey_to_addr, AppUser, SolPubkey, Target, TestError,
            TestResult,
        },
    },
    litesvm::types::TransactionMetadata,
    pinocchio::pubkey::Pubkey,
    solana_address::Address,
    solana_keypair::Keypair,
};

pub trait Token2022MemoTransferExtension {
    fn token_2022_try_enable_memo_transfer(
        &mut self,
        target: Target,
        token_account: &Pubkey,
        authority: &Pubkey,
        signer: AppUser,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_enable_memo_transfer_multisig(
        &mut self,
        target: Target,
        token_account: &Pubkey,
        multisig_authority: &Pubkey,
        signers: &[AppUser],
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_disable_memo_transfer(
        &mut self,
        target: Target,
        token_account: &Pubkey,
        authority: &Pubkey,
        signer: AppUser,
    ) -> TestResult<TransactionMetadata>;

    fn token_2022_try_disable_memo_transfer_multisig(
        &mut self,
        target: Target,
        token_account: &Pubkey,
        multisig_authority: &Pubkey,
        signers: &[AppUser],
    ) -> TestResult<TransactionMetadata>;
}

impl Token2022MemoTransferExtension for App {
    fn token_2022_try_enable_memo_transfer(
        &mut self,
        target: Target,
        token_account: &Pubkey,
        authority: &Pubkey,
        signer: AppUser,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&signer.keypair()];
        let authority_signers = &[&pin_pubkey_to_addr(&signer.pubkey().to_bytes())];

        let ix = spl_token_2022_interface::extension::memo_transfer::instruction::enable_required_transfer_memos(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(token_account),
            &pin_pubkey_to_addr(authority),
            authority_signers,
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

    fn token_2022_try_enable_memo_transfer_multisig(
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

        let (signers_keypairs, authority_signers) = extract_signers_and_authorities(signers);
        let authority_refs: Vec<_> = authority_signers.iter().collect();

        let ix = spl_token_2022_interface::extension::memo_transfer::instruction::enable_required_transfer_memos(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(token_account),
            &pin_pubkey_to_addr(multisig_authority),
            &authority_refs,
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
            &signers_keypairs,
            self.is_log_displayed,
        )
    }

    fn token_2022_try_disable_memo_transfer(
        &mut self,
        target: Target,
        token_account: &Pubkey,
        authority: &Pubkey,
        signer: AppUser,
    ) -> TestResult<TransactionMetadata> {
        let ProgramId {
            token_2022_program,
            token_2022_proxy,
            ..
        } = self.program_id;

        let signers = &[&signer.keypair()];
        let authority_signers = &[&pin_pubkey_to_addr(&signer.pubkey().to_bytes())];

        let ix = spl_token_2022_interface::extension::memo_transfer::instruction::disable_required_transfer_memos(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(token_account),
            &pin_pubkey_to_addr(authority),
            authority_signers,
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

    fn token_2022_try_disable_memo_transfer_multisig(
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

        let (signers_keypairs, authority_signers) = extract_signers_and_authorities(signers);
        let authority_refs: Vec<_> = authority_signers.iter().collect();

        let ix = spl_token_2022_interface::extension::memo_transfer::instruction::disable_required_transfer_memos(
            &token_2022_program.to_bytes().into(),
            &pin_pubkey_to_addr(token_account),
            &pin_pubkey_to_addr(multisig_authority),
            &authority_refs,
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
            &signers_keypairs,
            self.is_log_displayed,
        )
    }
}

////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////HELPERS/////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////

// dev : data[165..171] holds memo_transfer extention bytes; 165 = base token_account len, 171 = total with extention
#[derive(Debug, PartialEq, Eq)]
pub enum MemoStatus {
    /// [2, 0, 0, 0, 0, 0]
    /// Represents a MemoTransfer extension that has been initialized
    /// but not yet toggled to enforce memos on transfers.
    Initialized,

    /// [2, 8, 0, 1, 0, 1]
    /// Memo enforcement is active; transfers must include a memo instruction.
    Enabled,

    /// [2, 8, 0, 1, 0, 0]
    /// Memo enforcement disabled; transfers without memos are allowed.
    Disabled,

    /// [ Invalid Bytes ]
    /// Any other byte pattern, indicates data corruption or invalid state.
    Invalid,
}

impl MemoStatus {
    /// dev: just checks the 6-byte slice from [165..171] to figure out memo state
    pub fn check_memo_status(memo_data_slice: &[u8]) -> MemoStatus {
        match memo_data_slice {
            [2, 0, 0, 0, 0, 0] => MemoStatus::Initialized,
            [2, 8, 0, 1, 0, 1] => MemoStatus::Enabled,
            [2, 8, 0, 1, 0, 0] => MemoStatus::Disabled,
            _ => MemoStatus::Invalid,
        }
    }
}

/// Extracts signer keypairs and authority addresses from a list of AppUsers.
/// Returns a tuple `(signer_keypairs, authority_addresses)`.
fn extract_signers_and_authorities<'a>(signers: &'a [AppUser]) -> (Vec<Keypair>, Vec<Address>) {
    let mut signer_keypairs = Vec::new();
    let mut authority_addrs = Vec::new();

    for s in signers {
        signer_keypairs.push(s.keypair());
        authority_addrs.push(pin_pubkey_to_addr(&s.pubkey().to_bytes()));
    }

    (signer_keypairs, authority_addrs)
}
