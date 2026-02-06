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

/// Transfer, providing expected mint information and fees.
///
/// This instruction succeeds if the mint has no configured transfer fee
/// and the provided fee is 0. This allows applications to use
/// `TransferCheckedWithFee` with any mint.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The source account. May include the `TransferFeeAmount`
///      extension.
///   1. `[]` The token mint. May include the `TransferFeeConfig` extension.
///   2. `[writable]` The destination account. May include the
///      `TransferFeeAmount` extension.
///   3. `[signer]` The source account's owner/delegate.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The source account.
///   1. `[]` The token mint.
///   2. `[writable]` The destination account.
///   3. `[]` The source account's multisignature.
///   4. `..+N` `[]` The `N` signer accounts, where `N` is `1 <= N <= 11`.
pub struct TransferCheckedWithFee<'a, 'b, 'c> {
    /// The token program ID.
    pub token_program_id: &'b Address,

    /// The source account.
    pub source: &'a AccountView,

    /// The token mint.
    pub mint: &'a AccountView,

    /// The destination account.
    pub destination: &'a AccountView,

    /// The source account's owner/delegate or multisignature.
    pub authority: &'a AccountView,

    /// Multisignature owner/delegate.
    pub signers: &'c [&'a AccountView],

    /// The amount of tokens to transfer.
    pub amount: u64,

    /// Expected number of base 10 digits to the right of the decimal place.
    pub decimals: u8,

    /// Expected fee assessed on this transfer, calculated off-chain based
    /// on the `transfer_fee_basis_points` and `maximum_fee` of the mint.
    /// May be 0 for a mint without a configured transfer fee.
    pub fee: u64,
}

impl<'a, 'b, 'c> TransferCheckedWithFee<'a, 'b, 'c> {
    /// Instruction discriminator.
    pub const DISCRIMINATOR: u8 = 1;

    /// Creates a new `TransferCheckedWithFee` instruction
    /// with a single owner/delegate authority.
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub fn new(
        token_program_id: &'b Address,
        source: &'a AccountView,
        mint: &'a AccountView,
        destination: &'a AccountView,
        authority: &'a AccountView,
        amount: u64,
        decimals: u8,
        fee: u64,
    ) -> Self {
        Self {
            source,
            mint,
            destination,
            authority,
            signers: &[],
            amount,
            decimals,
            fee,
            token_program_id,
        }
    }

    /// Creates a new `TransferCheckedWithFee` instruction with a
    /// multisignature owner/delegate authority and signer accounts.
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub fn with_multisig(
        token_program_id: &'b Address,
        source: &'a AccountView,
        mint: &'a AccountView,
        destination: &'a AccountView,
        authority: &'a AccountView,
        signers: &'c [&'a AccountView],
        amount: u64,
        decimals: u8,
        fee: u64,
    ) -> Self {
        Self {
            source,
            mint,
            destination,
            authority,
            signers,
            amount,
            decimals,
            fee,
            token_program_id,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        // Instruction accounts.

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 4 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            // source account
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.source.address()));

            // mint account
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly(self.mint.address()));

            // destination account
            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::writable(self.destination.address()));

            // authority account (single or multisig)
            instruction_accounts
                .get_unchecked_mut(3)
                .write(InstructionAccount::new(
                    self.authority.address(),
                    false,
                    self.signers.is_empty(),
                ));

            // multisig signer accounts
            for (instruction_account, signer) in instruction_accounts
                .get_unchecked_mut(4..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // Instruction data.
        let mut instruction_data = [UNINIT_BYTE; 19];

        // discriminator
        write_bytes(
            &mut instruction_data[..2],
            &[
                ExtensionDiscriminator::TransferFee as u8,
                Self::DISCRIMINATOR,
            ],
        );
        // amount
        write_bytes(&mut instruction_data[2..10], &self.amount.to_le_bytes());
        // decimals
        unsafe { instruction_data.get_unchecked_mut(10).write(self.decimals) };
        // fee
        write_bytes(&mut instruction_data[11..19], &self.fee.to_le_bytes());

        // Instruction.

        let expected_accounts = 4 + signers.len();

        let instruction = InstructionView {
            program_id: self.token_program_id,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        // Accounts.

        let mut accounts = [UNINIT_ACCOUNT_REF; 4 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            // source account
            accounts.get_unchecked_mut(0).write(self.source);

            // mint account
            accounts.get_unchecked_mut(1).write(self.mint);

            // destination account
            accounts.get_unchecked_mut(2).write(self.destination);

            // authority account (single or multisig)
            accounts.get_unchecked_mut(3).write(self.authority);

            // multisig signer accounts
            for (account, signer) in accounts
                .get_unchecked_mut(4..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(*signer);
            }
        }

        invoke_signed_with_bounds::<{ 4 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
