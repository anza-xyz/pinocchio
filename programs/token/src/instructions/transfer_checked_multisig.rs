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

/// Transfer tokens from one Token account to another, validating the token
/// mint and decimals, where the authority is a multisig account.
///
/// ### Accounts:
///   0. `[WRITE]` The source account.
///   1. `[]` The token mint.
///   2. `[WRITE]` The destination account.
///   3. `[]` Authority account (multisig).
///   4. ..`4+N`. `[SIGNER]` The N signer accounts, where N is between 1 and 11.
pub struct TransferCheckedMultisig<'a, 'b>
where
    'a: 'b,
{
    /// Sender account.
    pub from: &'a AccountView,
    /// Mint Account.
    pub mint: &'a AccountView,
    /// Recipient account.
    pub to: &'a AccountView,
    /// Multisig authority account.
    pub multisig: &'a AccountView,
    /// Signer accounts.
    pub signers: &'b [&'a AccountView],
    /// Amount of micro-tokens to transfer.
    pub amount: u64,
    /// Decimal for the Token.
    pub decimals: u8,
}

impl TransferCheckedMultisig<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            from,
            mint,
            to,
            multisig,
            signers: multisig_signers,
            amount,
            decimals,
        } = self;

        if multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let num_accounts = 4 + multisig_signers.len();

        // Instruction accounts
        const UNINIT_INSTRUCTION_ACCOUNT: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts =
            [UNINIT_INSTRUCTION_ACCOUNT; 4 + MAX_MULTISIG_SIGNERS];

        unsafe {
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(from.address()));
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly(mint.address()));
            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::writable(to.address()));
            instruction_accounts
                .get_unchecked_mut(3)
                .write(InstructionAccount::readonly(multisig.address()));
        }

        for (instruction_account, signer) in
            instruction_accounts[4..].iter_mut().zip(multisig_signers.iter())
        {
            instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        // Instruction data layout:
        // - [0]: instruction discriminator (1 byte, u8)
        // - [1..9]: amount (8 bytes, u64)
        // - [9]: decimals (1 byte, u8)
        let mut instruction_data = [UNINIT_BYTE; 10];

        write_bytes(&mut instruction_data, &[12]);
        write_bytes(&mut instruction_data[1..9], &amount.to_le_bytes());
        write_bytes(&mut instruction_data[9..], &[decimals]);

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: unsafe {
                slice::from_raw_parts(instruction_accounts.as_ptr() as _, num_accounts)
            },
            data: unsafe { slice::from_raw_parts(instruction_data.as_ptr() as _, 10) },
        };

        // Account views
        const UNINIT_VIEW: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut acc_views = [UNINIT_VIEW; 4 + MAX_MULTISIG_SIGNERS];

        unsafe {
            acc_views.get_unchecked_mut(0).write(from);
            acc_views.get_unchecked_mut(1).write(mint);
            acc_views.get_unchecked_mut(2).write(to);
            acc_views.get_unchecked_mut(3).write(multisig);
        }

        for (account_view, signer) in acc_views[4..].iter_mut().zip(multisig_signers.iter()) {
            account_view.write(signer);
        }

        invoke_signed_with_bounds::<{ 4 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { slice::from_raw_parts(acc_views.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}
