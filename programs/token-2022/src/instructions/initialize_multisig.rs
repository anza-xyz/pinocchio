use core::{mem::MaybeUninit, slice};

use pinocchio::{
    account::AccountView,
    cpi::invoke_with_bounds,
    error::ProgramError,
    instruction::{AccountMeta, Instruction},
    Address, ProgramResult,
};

/// Maximum number of multisignature signers.
pub const MAX_MULTISIG_SIGNERS: usize = 11;

/// Initialize a new Multisig.
///
/// ### Accounts:
///   0. `[writable]` The multisig account to initialize.
///   1. `[]` Rent sysvar
///   2. `..+N` `[]` The `N` signer accounts, where `N` is `1 <=
///      N <= 11`.
pub struct InitializeMultisig<'a, 'b, 'c>
where
    'a: 'b,
{
    /// Multisig Account.
    pub multisig: &'a AccountView,
    /// Rent sysvar Account.
    pub rent_sysvar: &'a AccountView,
    /// Signer Accounts
    pub signers: &'b [&'a AccountView],
    /// The number of signers (M) required to validate this multisignature
    /// account.
    pub m: u8,
    /// Token Program.
    pub token_program: &'c Address,
}

impl InitializeMultisig<'_, '_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let &Self {
            multisig,
            rent_sysvar,
            signers,
            m,
            token_program,
        } = self;

        if signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let num_accounts = 2 + signers.len();

        // Account metadata
        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut acc_metas = [UNINIT_META; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_metas` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 and 1 are always present
            acc_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(multisig.address()));
            acc_metas
                .get_unchecked_mut(1)
                .write(AccountMeta::readonly(rent_sysvar.address()));
        }

        for (account_meta, signer) in acc_metas[2..].iter_mut().zip(signers.iter()) {
            account_meta.write(AccountMeta::readonly(signer.address()));
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: m (1 byte, u8)
        let data = &[2, m];

        let instruction = Instruction {
            program_id: token_program,
            accounts: unsafe { slice::from_raw_parts(acc_metas.as_ptr() as _, num_accounts) },
            data,
        };

        // Account info array
        const UNINIT_INFO: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut acc_infos = [UNINIT_INFO; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_infos` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 and 1 are always present
            acc_infos.get_unchecked_mut(0).write(multisig);
            acc_infos.get_unchecked_mut(1).write(rent_sysvar);
        }

        // Fill signer accounts
        for (account_info, signer) in acc_infos[2..].iter_mut().zip(signers.iter()) {
            account_info.write(signer);
        }

        invoke_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(&instruction, unsafe {
            slice::from_raw_parts(acc_infos.as_ptr() as _, num_accounts)
        })
    }
}
