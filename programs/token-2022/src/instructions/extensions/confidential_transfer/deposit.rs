use {
    crate::{
        instructions::{ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
        write_bytes, UNINIT_ACCOUNT_REF, UNINIT_BYTE, UNINIT_INSTRUCTION_ACCOUNT,
    },
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Deposit SPL Token into the pending balance of a confidential token
/// account.
///
/// The account owner can then invoke the `ApplyPendingBalance` instruction
/// to roll the deposit into their available balance at a time of their
/// choosing.
///
/// Fails if the source or destination accounts are frozen.
/// Fails if the associated mint is extended as `NonTransferable`.
/// Fails if the associated mint is extended as `ConfidentialMintBurn`.
/// Fails if the associated mint is extended as `Pausable` extension.
///
/// Accounts expected by this instruction:
///
/// * Single owner/delegate
/// 0. `[writable]` The SPL Token account.
/// 1. `[]` The Token mint.
/// 2. `[signer]` The single account owner or delegate.
///
/// * Multisignature owner/delegate
/// 0. `[writable]` The SPL Token account.
/// 1. `[]` The token mint.
/// 2. `[]` The multisig account owner or delegate.
/// 3. ...`[signer]` Required M signer accounts for the SPL Token Multisig
///    account.
pub struct Deposit<'a, 'b> {
    /// The token account
    pub token_account: &'a AccountView,
    /// The token mint
    pub mint: &'a AccountView,
    /// The token account authority
    pub owner: &'a AccountView,
    /// The multisig signers
    pub signers: &'b [&'a AccountView],
    /// The token program
    pub token_program: &'a Address,

    /// Expected data
    ///
    /// The amount of tokens to deposit
    pub amount: u64,
    /// Expected number of base 10 digits to the right of the decimal place
    pub decimals: u8,
}

impl Deposit<'_, '_> {
    const DISCRIMINATOR: u8 = 5;

    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        // Instruction accounts

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 3 + MAX_MULTISIG_SIGNERS];

        // SAFETY: allocation is valid to the maximum number of accounts
        unsafe {
            // token account
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.token_account.address()));

            // token mint
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly(self.mint.address()));

            // owner
            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::new(
                    self.owner.address(),
                    false,
                    signers.is_empty(),
                ));

            // multisig signers
            for (account, signer) in instruction_accounts
                .get_unchecked_mut(3..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        let mut instruction_data = [UNINIT_BYTE; 2 + 8 + 1];

        // discriminators
        write_bytes(
            &mut instruction_data[..2],
            &[
                ExtensionDiscriminator::ConfidentialTransfer as u8,
                Deposit::DISCRIMINATOR,
            ],
        );

        // amount
        write_bytes(&mut instruction_data[2..10], &self.amount.to_le_bytes());

        // decimals
        unsafe {
            instruction_data.get_unchecked_mut(10).write(self.decimals);
        }

        // Instruction

        let expected_accounts = 3 + self.signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        // Accounts

        let mut accounts = [UNINIT_ACCOUNT_REF; 3 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // token account
            accounts.get_unchecked_mut(0).write(self.token_account);

            // token mint
            accounts.get_unchecked_mut(1).write(self.mint);

            // owner
            accounts.get_unchecked_mut(2).write(self.owner);

            // multisig signers
            for (account, signer) in accounts
                .get_unchecked_mut(3..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(*signer);
            }
        }

        invoke_signed_with_bounds::<{ 3 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
