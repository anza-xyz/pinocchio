use {
    crate::{instructions::MAX_MULTISIG_SIGNERS, write_bytes, UNINIT_BYTE},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Transfer lamports from a native SOL account to a destination account.
///
/// This is useful to unwrap lamports from a wrapped SOL account.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The source account.
///   1. `[writable]` The destination account.
///   2. `[signer]` The source account's owner/delegate.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The source account.
///   1. `[writable]` The destination account.
///   2. `[]` The source account's multisignature owner/delegate.
///   3. `..+M` `[signer]` M signer accounts.
pub struct UnwrapLamports<'a, 'b, 'c> {
    /// The source account.
    pub account: &'a AccountView,

    /// The destination account.
    pub destination: &'a AccountView,

    /// The owner/delegate account.
    pub authority: &'a AccountView,

    /// Multisignature owner/delegate.
    pub signers: &'c [&'a AccountView],

    /// The amount of lamports to transfer.
    pub amount: Option<u64>,

    /// Token Program
    pub token_program: &'b Address,
}

impl<'a, 'b, 'c> UnwrapLamports<'a, 'b, 'c> {
    pub const DISCRIMINATOR: u8 = 45;

    /// Creates a new `UnwrapLamports` instruction with a single
    /// owner/delegate authority.
    #[inline(always)]
    pub fn new(
        token_program: &'b Address,
        account: &'a AccountView,
        destination: &'a AccountView,
        authority: &'a AccountView,
        amount: Option<u64>,
    ) -> Self {
        Self::with_signers(token_program, account, destination, authority, amount, &[])
    }

    /// Creates a new `UnwrapLamports` instruction with a
    /// multisignature owner/delegate authority and signer accounts.
    #[inline(always)]
    pub fn with_signers(
        token_program: &'b Address,
        account: &'a AccountView,
        destination: &'a AccountView,
        authority: &'a AccountView,
        amount: Option<u64>,
        signers: &'c [&'a AccountView],
    ) -> Self {
        Self {
            account,
            destination,
            authority,
            signers,
            amount,
            token_program,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.signers.len() > MAX_MULTISIG_SIGNERS {
            Err(ProgramError::InvalidArgument)?;
        }

        let expected_accounts = 3 + self.signers.len();

        // Instruction accounts.

        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 3 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.account.address()));

            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::writable(self.destination.address()));

            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::new(
                    self.authority.address(),
                    false,
                    self.signers.is_empty(),
                ));

            for (account, signer) in instruction_accounts
                .get_unchecked_mut(3..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // Accounts.

        const UNINIT_INFO: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut accounts = [UNINIT_INFO; 3 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            accounts.get_unchecked_mut(0).write(self.account);

            accounts.get_unchecked_mut(1).write(self.destination);

            accounts.get_unchecked_mut(2).write(self.authority);

            for (account, signer) in accounts
                .get_unchecked_mut(3..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(signer);
            }
        }

        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 10];
        let mut expected_data = 2;

        // discriminator
        instruction_data[0].write(Self::DISCRIMINATOR);
        // amount
        if let Some(amount) = self.amount {
            instruction_data[1].write(1);
            write_bytes(&mut instruction_data[2..10], &amount.to_le_bytes());
            expected_data += 8;
        } else {
            instruction_data[1].write(0);
        }

        invoke_signed_with_bounds::<{ 3 + MAX_MULTISIG_SIGNERS }>(
            &InstructionView {
                program_id: self.token_program,
                // SAFETY: instruction accounts have `expected_accounts` initialized.
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                // SAFETY: instruction data has `expected_data` initialized.
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, expected_data) },
            },
            // SAFETY: accounts have `expected_accounts` initialized.
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
