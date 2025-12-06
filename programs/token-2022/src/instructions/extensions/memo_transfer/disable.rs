use {
    crate::{instructions::extensions::ExtensionDiscriminator, instructions::MAX_MULTISIG_SIGNERS},
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

/// Disable the Memo-Transfer extension on a token account.
///
/// Expected accounts:
///
/// **Single authority**
/// 0. `[writable]` The token account to disable memo transfer.
/// 1. `[signer]` The owner of the token account.
///
/// **Multisignature authority**
/// 0. `[writable]` The token account to disable memo transfer.
/// 1. `[readonly]` The multisig account that owns the token account.
/// 2. `[signer]` M signer accounts (as required by the multisig).
pub struct Disable<'a, 'b, 'c> {
    /// The token account to disable with the Memo-Transfer extension.
    pub token_account: &'a AccountInfo,
    /// The owner of the token account (single or multisig).
    pub authority: &'a AccountInfo,
    /// Signer accounts if the authority is a multisig.
    pub signers: &'c [&'a AccountInfo],
    /// Token program (Token-2022).
    pub token_program: &'b Pubkey,
}

impl Disable<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 1;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            token_account,
            authority,
            signers: multisig_accounts,
            token_program,
            ..
        } = self;

        if multisig_accounts.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        const UNINIT_ACCOUNT_METAS: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut account_metas = [UNINIT_ACCOUNT_METAS; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_metas` is sized to 2 + MAX_MULTISIG_SIGNERS

            // - Index 0 is always present (TokenAccount)
            account_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(token_account.key()));

            // - Index 1 is always present (Authority)
            account_metas.get_unchecked_mut(1).write(AccountMeta::new(
                authority.key(),
                false,
                multisig_accounts.is_empty(),
            ));
        }

        for (account_meta, signer) in account_metas[2..].iter_mut().zip(multisig_accounts.iter()) {
            account_meta.write(AccountMeta::readonly_signer(signer.key()));
        }

        let data = &[
            ExtensionDiscriminator::MemoTransfer as u8,
            Self::DISCRIMINATOR,
        ];

        let num_accounts = 2 + multisig_accounts.len();

        let instruction = Instruction {
            program_id: token_program,
            data,
            accounts: unsafe { slice::from_raw_parts(account_metas.as_ptr() as _, num_accounts) },
        };

        // Account info array
        const UNINIT_ACCOUNT_INFOS: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();
        let mut account_infos = [UNINIT_ACCOUNT_INFOS; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_infos` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            account_infos.get_unchecked_mut(0).write(token_account);
            // - Index 1 is always present
            account_infos.get_unchecked_mut(1).write(authority);
        }

        // Fill signer accounts
        for (account_info, signer) in account_infos[2..].iter_mut().zip(multisig_accounts.iter()) {
            account_info.write(signer);
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe {
                slice::from_raw_parts(account_infos.as_ptr() as *const &AccountInfo, num_accounts)
            },
            signers,
        )
    }
}
