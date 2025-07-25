use core::{mem::MaybeUninit, slice};

use pinocchio::{
    account_info::AccountInfo,
    cpi::slice_invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    ProgramResult,
};

/// Initialize a new Multisig.
///
/// ### Accounts:
///   0. `[writable]` The multisig account to initialize.
///   1. `[]` Rent sysvar
///   2. ..`2+N`. `[]` The N signer accounts, where N is between 1 and 11.
pub struct InitializeMultisig<'a> {
    /// Multisig Account.
    pub multisig: &'a AccountInfo,
    /// Rent sysvar Account.
    pub rent_sysvar: &'a AccountInfo,
    /// Signer Accounts
    pub multisig_signers: &'a [&'a AccountInfo],
    /// The number of signers (M) required to validate this multisignature
    /// account.
    pub m: u8,
}

impl InitializeMultisig<'_> {
    pub const MAX_ALLOWED_ACCOUNTS: usize = 1 + 1 + 11; // 1 multisig + 1 rent_sysvar + 11 MAX_SIGNERS

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.slice_invoke_signed(&[])
    }

    pub fn slice_invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            multisig,
            rent_sysvar,
            multisig_signers,
            m,
        } = self;

        // MAX_SIGNERS = 11
        if multisig_signers.len() > 11 {
            return Err(ProgramError::InvalidArgument);
        }

        let num_accounts = 2 + multisig_signers.len();

        // Account metadata
        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut acc_metas = [UNINIT_META; Self::MAX_ALLOWED_ACCOUNTS];

        unsafe {
            // SAFETY:
            // - `account_metas` is sized to at least MAX_ALLOWED_ACCOUNTS
            // - Index 0 and 1 are always present
            acc_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(multisig.key()));
            acc_metas
                .get_unchecked_mut(1)
                .write(AccountMeta::readonly(rent_sysvar.key()));
        }

        for i in 2..(2 + multisig_signers.len()) {
            unsafe {
                // SAFETY:
                // - `i` in 2..(2 + multisig_signers.len()) is guaranteed less than MAX_ALLOWED_ACCOUNTS
                // - `i - 2` < multisig_signers.len()
                acc_metas.get_unchecked_mut(i).write(AccountMeta::readonly(
                    multisig_signers.get_unchecked(i - 2).key(),
                ));
            }
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: m (1 byte, u8)
        let data = &[2, m];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: unsafe { slice::from_raw_parts(acc_metas.as_ptr() as _, num_accounts) },
            data,
        };

        // Account info array
        const UNINIT_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();
        let mut acc_infos = [UNINIT_INFO; Self::MAX_ALLOWED_ACCOUNTS];

        unsafe {
            // SAFETY:
            // - `account_infos` is sized to at least MAX_ALLOWED_ACCOUNTS
            // - Index 0 and 1 are always present
            acc_infos.get_unchecked_mut(0).write(multisig);
            acc_infos.get_unchecked_mut(1).write(rent_sysvar);
        }

        // Fill signer accounts
        for i in 2..(2 + multisig_signers.len()) {
            unsafe {
                // SAFETY:
                // - `i` in 2..(2 + multisig_signers.len()) is guaranteed less than MAX_ALLOWED_ACCOUNTS
                // - `i - 2` < multisig_signers.len()
                acc_infos
                    .get_unchecked_mut(i)
                    .write(multisig_signers.get_unchecked(i - 2));
            }
        }

        slice_invoke_signed(
            &instruction,
            unsafe { slice::from_raw_parts(acc_infos.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}
