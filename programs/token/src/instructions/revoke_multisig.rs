use {
    crate::instructions::MAX_MULTISIG_SIGNERS,
    core::{mem::MaybeUninit, slice},
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Revokes the delegate's authority where the owner is a multisig account.
///
/// ### Accounts:
///   0. `[WRITE]` The source account.
///   1. `[]` Authority account (multisig).
///   2. ..`2+N`. `[SIGNER]` The N signer accounts, where N is between 1 and 11.
pub struct RevokeMultisig<'a, 'b>
where
    'a: 'b,
{
    /// Source Account.
    pub source: &'a AccountView,
    /// Multisig authority account.
    pub multisig: &'a AccountView,
    /// Signer accounts.
    pub signers: &'b [&'a AccountView],
}

impl RevokeMultisig<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            source,
            multisig,
            signers: multisig_signers,
        } = self;

        if multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let num_accounts = 2 + multisig_signers.len();

        const UNINIT_INSTRUCTION_ACCOUNT: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts =
            [UNINIT_INSTRUCTION_ACCOUNT; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(source.address()));
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly(multisig.address()));
        }

        for (instruction_account, signer) in
            instruction_accounts[2..].iter_mut().zip(multisig_signers.iter())
        {
            instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: unsafe {
                slice::from_raw_parts(instruction_accounts.as_ptr() as _, num_accounts)
            },
            data: &[5],
        };

        const UNINIT_VIEW: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut acc_views = [UNINIT_VIEW; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            acc_views.get_unchecked_mut(0).write(source);
            acc_views.get_unchecked_mut(1).write(multisig);
        }

        for (account_view, signer) in acc_views[2..].iter_mut().zip(multisig_signers.iter()) {
            account_view.write(signer);
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { slice::from_raw_parts(acc_views.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}
