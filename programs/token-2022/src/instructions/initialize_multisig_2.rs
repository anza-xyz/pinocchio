use core::{mem::MaybeUninit, slice};

use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::{cpi::invoke_with_bounds, AccountRole, InstructionView};
use solana_program_error::{ProgramError, ProgramResult};

use crate::instructions::MAX_MULTISIG_SIGNERS;

/// Initialize a new Multisig.
///
/// ### Accounts:
///   0. `[writable]` The multisig account to initialize.
///   1. `..+N` `[]` The `N` signer accounts, where `N` is `1 <=
///      N <= 11`.
pub struct InitializeMultisig2<'a, 'b, 'c>
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
    /// Token Program.
    pub token_program: &'c Address,
}

impl InitializeMultisig2<'_, '_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let &Self {
            multisig,
            signers,
            m,
            token_program,
        } = self;

        if signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let num_accounts = 1 + signers.len();

        // Account metadata
        const UNINIT_META: MaybeUninit<AccountRole> = MaybeUninit::<AccountRole>::uninit();
        let mut acc_metas = [UNINIT_META; 1 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_metas` is sized to 1 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            acc_metas
                .get_unchecked_mut(0)
                .write(AccountRole::writable(multisig.address()));
        }

        for (account_meta, signer) in acc_metas[1..].iter_mut().zip(signers.iter()) {
            account_meta.write(AccountRole::readonly(signer.address()));
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: m (1 byte, u8)
        let data = &[19, m];

        let instruction = InstructionView {
            program_id: token_program,
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
