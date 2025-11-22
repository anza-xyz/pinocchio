use core::{mem::MaybeUninit, slice};

use pinocchio::{
    account::AccountView,
    cpi::invoke_with_bounds,
    error::ProgramError,
    instruction::{AccountMeta, Instruction},
    ProgramResult,
};

use crate::instructions::MAX_MULTISIG_SIGNERS;

/// Initialize a new Multisig.
///
/// ### Accounts:
///   0. `[writable]` The multisig account to initialize.
///   1. ..`1+N`. `[]` The N signer accounts, where N is between 1 and 11.
pub struct InitializeMultisig2<'a, 'b>
where
    'a: 'b,
{
    /// Multisig Account.
    pub multisig: &'a AccountView,
    /// Signer Accounts
    pub signers: &'b [&'a AccountView],
    /// The number of signers (M) required to validate this multisignature
    /// account.
    pub m: u8,
}

impl InitializeMultisig2<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let &Self {
            multisig,
            signers,
            m,
        } = self;

        if signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let num_accounts = 1 + signers.len();

        // Account metadata
        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut acc_metas = [UNINIT_META; 1 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_metas` is sized to 1 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            acc_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(multisig.address()));
        }

        for (account_meta, signer) in acc_metas[1..].iter_mut().zip(signers.iter()) {
            account_meta.write(AccountMeta::readonly(signer.address()));
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: m (1 byte, u8)
        let data = &[19, m];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: unsafe { slice::from_raw_parts(acc_metas.as_ptr() as _, num_accounts) },
            data,
        };

        // Account info array
        const UNINIT_INFO: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut acc_infos = [UNINIT_INFO; 1 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_infos` is sized to 1 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            acc_infos.get_unchecked_mut(0).write(multisig);
        }

        // Fill signer accounts
        for (account_info, signer) in acc_infos[1..].iter_mut().zip(signers.iter()) {
            account_info.write(signer);
        }

        invoke_with_bounds::<{ 1 + MAX_MULTISIG_SIGNERS }>(&instruction, unsafe {
            slice::from_raw_parts(acc_infos.as_ptr() as _, num_accounts)
        })
    }
}
