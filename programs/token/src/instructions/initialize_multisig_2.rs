use core::{mem::MaybeUninit, slice};

use solana_account_view::AccountView;
use solana_instruction_view::{cpi::invoke_with_bounds, InstructionAccount, InstructionView};
use solana_program_error::{ProgramError, ProgramResult};

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

        // Instruction accounts
        const UNINIT_INSTRUCTION_ACCOUNT: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 1 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `instruction_accounts` is sized to 1 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(multisig.address()));
        }

        for (instruction_account, signer) in
            instruction_accounts[1..].iter_mut().zip(signers.iter())
        {
            instruction_account.write(InstructionAccount::readonly(signer.address()));
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: m (1 byte, u8)
        let data = &[19, m];

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: unsafe {
                slice::from_raw_parts(instruction_accounts.as_ptr() as _, num_accounts)
            },
            data,
        };

        // Account view array
        const UNINIT_VIEW: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut acc_views = [UNINIT_VIEW; 1 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_views` is sized to 1 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            acc_views.get_unchecked_mut(0).write(multisig);
        }

        // Fill signer accounts
        for (account_view, signer) in acc_views[1..].iter_mut().zip(signers.iter()) {
            account_view.write(signer);
        }

        invoke_with_bounds::<{ 1 + MAX_MULTISIG_SIGNERS }>(&instruction, unsafe {
            slice::from_raw_parts(acc_views.as_ptr() as _, num_accounts)
        })
    }
}
