use core::{mem::MaybeUninit, slice};

use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::{cpi::invoke_with_bounds, InstructionAccount, InstructionView};
use solana_program_error::{ProgramError, ProgramResult};

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

        // Instruction accounts
        const UNINIT_INSTRUCTION_ACCOUNT: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `instruction_accounts` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 and 1 are always present
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(multisig.address()));
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly(rent_sysvar.address()));
        }

        for (instruction_account, signer) in
            instruction_accounts[2..].iter_mut().zip(signers.iter())
        {
            instruction_account.write(InstructionAccount::readonly(signer.address()));
        }

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: m (1 byte, u8)
        let data = &[2, m];

        let instruction = InstructionView {
            program_id: token_program,
            accounts: unsafe {
                slice::from_raw_parts(instruction_accounts.as_ptr() as _, num_accounts)
            },
            data,
        };

        // Account view array
        const UNINIT_VIEW: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut acc_views = [UNINIT_VIEW; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_views` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 and 1 are always present
            acc_views.get_unchecked_mut(0).write(multisig);
            acc_views.get_unchecked_mut(1).write(rent_sysvar);
        }

        // Fill signer accounts
        for (account_view, signer) in acc_views[2..].iter_mut().zip(signers.iter()) {
            account_view.write(signer);
        }

        invoke_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(&instruction, unsafe {
            slice::from_raw_parts(acc_views.as_ptr() as _, num_accounts)
        })
    }
}
