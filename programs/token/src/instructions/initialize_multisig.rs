use core::{mem::MaybeUninit, slice};

use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_with_bounds,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    ProgramResult,
};

/// Maximum number of multisignature signers.
pub const MAX_MULTISIG_SIGNERS: usize = 11;

/// Initialize a new Multisig.
///
/// ### Accounts:
///   0. `[writable]` The multisig account to initialize.
///   1. `[]` Rent sysvar
///   2. ..`2+N`. `[]` The N signer accounts, where N is between 1 and 11.
pub struct InitializeMultisig<'a, 'b>
where
    'a: 'b,
{
    /// Multisig Account.
    pub multisig: &'a AccountInfo,
    /// Rent sysvar Account.
    pub rent_sysvar: &'a AccountInfo,
    /// Signer Accounts
    pub signers: &'b [&'a AccountInfo],
    /// The number of signers (M) required to validate this multisignature
    /// account.
    pub m: u8,
}

impl InitializeMultisig<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let &Self {
            multisig,
            rent_sysvar,
            signers,
            m,
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
                .write(AccountMeta::writable(multisig.key()));
            acc_metas
                .get_unchecked_mut(1)
                .write(AccountMeta::readonly(rent_sysvar.key()));
        }

        for (account_meta, signer) in acc_metas[2..].iter_mut().zip(signers.iter()) {
            account_meta.write(AccountMeta::readonly(signer.key()));
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
        let mut acc_infos = [const { MaybeUninit::uninit() }; 2 + MAX_MULTISIG_SIGNERS];

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
