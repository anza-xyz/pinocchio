use {
    crate::{
        extension::interest_bearing_mint::state::interest_bearing_mint_update_rate_instruction_data,
        instructions::MAX_MULTISIG_SIGNERS,
    },
    core::{mem::MaybeUninit, slice},
    pinocchio::{
        account_info::AccountInfo,
        cpi::{invoke_signed, invoke_with_bounds},
        instruction::{AccountMeta, Instruction, Signer},
        program_error::ProgramError,
        pubkey::Pubkey,
        ProgramResult,
    },
};

/// Update the interest rate. Only supported for mints that
/// include the `InterestBearingConfig` extension.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The mint.
///   1. `[signer]` The mint rate authority.
///
///   * Multisignature authority
///   0. `[writable]` The mint.
///   1. `[]` The mint's multisignature rate authority.
///   2. `..2+M` `[signer]` M signer accounts.
pub struct UpdateRate<'a, 'b> {
    /// Mint Account
    pub mint: &'a AccountInfo,
    /// The rate authority.
    pub authority: &'a AccountInfo,
    /// The new interest rate
    pub rate: i16,
    /// The Signer accounts if `authority` is a multisig
    pub signers: &'b [AccountInfo],
    /// Token Program
    pub token_program: &'b Pubkey,
}

impl UpdateRate<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let is_multisig = !self.signers.is_empty();

        if is_multisig {
            self.invoke_multisig()
        } else {
            self.invoke_single_owner(signers)
        }
    }

    #[inline(always)]
    fn invoke_single_owner(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            mint,
            authority,
            rate,
            token_program,
            ..
        } = self;

        let account_metas = [
            AccountMeta::writable(mint.key()),
            AccountMeta::readonly_signer(authority.key()),
        ];

        let data = interest_bearing_mint_update_rate_instruction_data(rate);

        let instruction = Instruction {
            accounts: &account_metas,
            data: unsafe { slice::from_raw_parts(data.as_ptr() as _, data.len()) },
            program_id: token_program,
        };

        invoke_signed(&instruction, &[mint, authority], signers)
    }

    #[inline(always)]
    fn invoke_multisig(&self) -> ProgramResult {
        let &Self {
            mint,
            authority,
            rate,
            signers: multisig_signers,
            token_program,
        } = self;
        
        if multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let num_accounts = 2 + multisig_signers.len();

        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut acc_metas = [UNINIT_META; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY
            acc_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(mint.key()));
            acc_metas
                .get_unchecked_mut(1)
                .write(AccountMeta::readonly(authority.key()));
        }

        for (account_meta, signer) in acc_metas[2..].iter_mut().zip(multisig_signers.iter()) {
            account_meta.write(AccountMeta::readonly_signer(signer.key()));
        }

        let data = interest_bearing_mint_update_rate_instruction_data(rate);

        let instruction = Instruction {
            accounts: unsafe { slice::from_raw_parts(acc_metas.as_ptr() as _, num_accounts) },
            data: unsafe { slice::from_raw_parts(data.as_ptr() as _, data.len()) },
            program_id: token_program,
        };

        const UNINIT_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();
        let mut acc_infos = [UNINIT_INFO; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY
            acc_infos.get_unchecked_mut(0).write(mint);
            acc_infos.get_unchecked_mut(1).write(authority);
        }

        for (account_info, signer) in acc_infos[2..].iter_mut().zip(multisig_signers.iter()) {
            account_info.write(signer);
        }

        invoke_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(&instruction, unsafe {
            slice::from_raw_parts(acc_infos.as_ptr() as _, num_accounts)
        })
    }
}

