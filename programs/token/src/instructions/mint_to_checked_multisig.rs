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

/// Mints new tokens to an account, validating the token mint and decimals,
/// where the mint authority is a multisig account.
///
/// ### Accounts:
///   0. `[WRITE]` The mint.
///   1. `[WRITE]` The account to mint tokens to.
///   2. `[]` The mint's minting authority (multisig).
///   3. ..`3+N`. `[SIGNER]` The N signer accounts, where N is between 1 and 11.
pub struct MintToCheckedMultisig<'a, 'b>
where
    'a: 'b,
{
    /// Mint Account.
    pub mint: &'a AccountView,
    /// Token Account.
    pub account: &'a AccountView,
    /// Multisig mint authority account.
    pub multisig: &'a AccountView,
    /// Signer accounts.
    pub signers: &'b [&'a AccountView],
    /// Amount.
    pub amount: u64,
    /// Decimals.
    pub decimals: u8,
}

impl MintToCheckedMultisig<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            mint,
            account,
            multisig,
            signers: multisig_signers,
            amount,
            decimals,
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
                .write(InstructionAccount::writable(mint.address()));
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::writable(account.address()));
            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::readonly(multisig.address()));
        }

        for (instruction_account, signer) in
            instruction_accounts[3..].iter_mut().zip(multisig_signers.iter())
        {
            instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        let mut instruction_data = [UNINIT_BYTE; 10];

        write_bytes(&mut instruction_data, &[14]);
        write_bytes(&mut instruction_data[1..9], &amount.to_le_bytes());
        write_bytes(&mut instruction_data[9..], &[decimals]);

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: unsafe {
                slice::from_raw_parts(instruction_accounts.as_ptr() as _, num_accounts)
            },
            data: unsafe { slice::from_raw_parts(instruction_data.as_ptr() as _, 10) },
        };

        const UNINIT_VIEW: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut acc_views = [UNINIT_VIEW; 3 + MAX_MULTISIG_SIGNERS];

        unsafe {
            acc_views.get_unchecked_mut(0).write(mint);
            acc_views.get_unchecked_mut(1).write(account);
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
