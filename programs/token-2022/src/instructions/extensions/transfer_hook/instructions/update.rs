use {
    crate::{
        instructions::extensions::{
            transfer_hook::state::{
                offset_transfer_hook_update as OFFSET, TransferHookInstruction,
            },
            ExtensionDiscriminator,
        },
        instructions::MAX_MULTISIG_SIGNERS,
    },
    core::{mem::MaybeUninit, slice},
    pinocchio::{
        account_info::AccountInfo,
        cpi::invoke_signed_with_bounds,
        instruction::{AccountMeta, Instruction, Signer},
        program_error::ProgramError,
        pubkey::Pubkey,
        ProgramResult,
    },
};

pub struct UpdateTransferHook<'a, 'b> {
    /// Mint Account to update.
    pub mint_account: &'a AccountInfo,
    /// Authority Account.
    pub authority: &'a AccountInfo,
    /// Signer Accounts (for multisig support)
    pub signers: &'a [AccountInfo],
    /// Program that authorizes the transfer
    pub program_id: Option<&'b Pubkey>,
    /// Token Program
    pub token_program: &'b Pubkey,
}

impl UpdateTransferHook<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            mint_account,
            authority,
            signers: account_signers,
            token_program,
            ..
        } = self;

        if account_signers.len() > MAX_MULTISIG_SIGNERS {
            Err(ProgramError::InvalidArgument)?;
        }

        let num_accounts = 2 + account_signers.len();

        // Account metadata
        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut acc_metas = [UNINIT_META; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_metas` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            acc_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(mint_account.key()));
            // - Index 1 is always present
            if account_signers.is_empty() {
                acc_metas
                    .get_unchecked_mut(1)
                    .write(AccountMeta::readonly_signer(authority.key()));
            } else {
                acc_metas
                    .get_unchecked_mut(1)
                    .write(AccountMeta::readonly(authority.key()));
            }
        }

        for (account_meta, signer) in acc_metas[2..].iter_mut().zip(account_signers.iter()) {
            account_meta.write(AccountMeta::readonly_signer(signer.key()));
        }

        let mut buffer = [0u8; 34];
        let data = update_instruction_data(&mut buffer, self.program_id);

        let instruction = Instruction {
            program_id: token_program,
            accounts: unsafe { slice::from_raw_parts(acc_metas.as_ptr() as _, num_accounts) },
            data,
        };

        // Account info array
        const UNINIT_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();
        let mut acc_infos = [UNINIT_INFO; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_infos` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            acc_infos.get_unchecked_mut(0).write(mint_account);
            // - Index 1 is always present
            acc_infos.get_unchecked_mut(1).write(authority);
        }

        // Fill signer accounts
        for (account_info, signer) in acc_infos[2..].iter_mut().zip(account_signers.iter()) {
            account_info.write(signer);
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { slice::from_raw_parts(acc_infos.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}

#[inline(always)]
fn update_instruction_data<'a>(buffer: &'a mut [u8], program_id: Option<&Pubkey>) -> &'a [u8] {
    let offset = OFFSET::START as usize;

    // Set discriminators (TransferHook + Update)
    buffer[..offset].copy_from_slice(&[
        ExtensionDiscriminator::TransferHook as u8,
        TransferHookInstruction::Update as u8,
    ]);

    // Set program_id at offset [2..34]
    if let Some(x) = program_id {
        buffer[offset..OFFSET::END as usize].copy_from_slice(x);
    } else {
        buffer[offset..OFFSET::END as usize].copy_from_slice(&[0; 32]);
    }

    buffer
}
