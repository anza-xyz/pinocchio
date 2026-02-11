use {
    crate::{instructions::MAX_MULTISIG_SIGNERS, write_bytes, UNINIT_BYTE},
    core::{mem::MaybeUninit, slice},
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Approves a delegate where the authority is a multisig account.
///
/// ### Accounts:
///   0. `[WRITE]` The token account.
///   1. `[]` The delegate.
///   2. `[]` Authority account (multisig).
///   3. ..`3+N`. `[SIGNER]` The N signer accounts, where N is between 1 and 11.
pub struct ApproveMultisig<'a, 'b>
where
    'a: 'b,
{
    /// Source Account.
    pub source: &'a AccountView,
    /// Delegate Account.
    pub delegate: &'a AccountView,
    /// Multisig authority account.
    pub multisig: &'a AccountView,
    /// Signer accounts.
    pub signers: &'b [&'a AccountView],
    /// Amount.
    pub amount: u64,
}

impl ApproveMultisig<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            source,
            delegate,
            multisig,
            signers: multisig_signers,
            amount,
        } = self;

        if multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let num_accounts = 3 + multisig_signers.len();

        const UNINIT_INSTRUCTION_ACCOUNT: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts =
            [UNINIT_INSTRUCTION_ACCOUNT; 3 + MAX_MULTISIG_SIGNERS];

        unsafe {
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(source.address()));
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly(delegate.address()));
            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::readonly(multisig.address()));
        }

        for (instruction_account, signer) in
            instruction_accounts[3..].iter_mut().zip(multisig_signers.iter())
        {
            instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        let mut instruction_data = [UNINIT_BYTE; 9];

        write_bytes(&mut instruction_data, &[4]);
        write_bytes(&mut instruction_data[1..], &amount.to_le_bytes());

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: unsafe {
                slice::from_raw_parts(instruction_accounts.as_ptr() as _, num_accounts)
            },
            data: unsafe { slice::from_raw_parts(instruction_data.as_ptr() as _, 9) },
        };

        const UNINIT_VIEW: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut acc_views = [UNINIT_VIEW; 3 + MAX_MULTISIG_SIGNERS];

        unsafe {
            acc_views.get_unchecked_mut(0).write(source);
            acc_views.get_unchecked_mut(1).write(delegate);
            acc_views.get_unchecked_mut(2).write(multisig);
        }

        for (account_view, signer) in acc_views[3..].iter_mut().zip(multisig_signers.iter()) {
            account_view.write(signer);
        }

        invoke_signed_with_bounds::<{ 3 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { slice::from_raw_parts(acc_views.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}
