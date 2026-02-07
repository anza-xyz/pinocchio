use {
    crate::{
        instructions::{ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
        UNINIT_ACCOUNT_REF, UNINIT_INSTRUCTION_ACCOUNT,
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

/// Set transfer fee. Only supported for mints that include the
/// `TransferFeeConfig` extension.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The mint.
///   1. `[signer]` The mint's fee account owner.
///
///   * Multisignature authority
///   0. `[writable]` The mint.
///   1. `[]` The mint's multisignature fee account owner.
///   2. `..2+M` `[signer]` M signer accounts.
pub struct SetTransferFee<'a, 'b, 'c> {
    /// The token mint.
    pub mint: &'a AccountView,

    /// The mint's `withdraw_withheld_authority` or multisig.
    pub authority: &'a AccountView,

    /// Multisignature owner/delegate.
    pub signers: &'c [&'a AccountView],

    /// Amount of transfer collected as fees, expressed as basis points of
    /// the transfer amount
    pub transfer_fee_basis_points: u16,

    /// Maximum fee assessed on transfers
    pub maximum_fee: u64,

    /// The token program.
    pub token_program: &'b Address,
}

impl<'a, 'b, 'c> SetTransferFee<'a, 'b, 'c> {
    pub const DISCRIMINATOR: u8 = 5;

    /// Creates a new `SetTransferFee` instruction
    /// with a single owner/delegate authority.
    #[inline(always)]
    pub fn new(
        token_program: &'b Address,
        mint: &'a AccountView,
        authority: &'a AccountView,
        transfer_fee_basis_points: u16,
        maximum_fee: u64,
    ) -> Self {
        Self {
            mint,
            authority,
            signers: &[],
            transfer_fee_basis_points,
            maximum_fee,
            token_program,
        }
    }

    /// Creates a new `SetTransferFee` instruction with a
    /// multisignature owner/delegate authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig(
        token_program: &'b Address,
        mint: &'a AccountView,
        authority: &'a AccountView,
        signers: &'c [&'a AccountView],
        transfer_fee_basis_points: u16,
        maximum_fee: u64,
    ) -> Self {
        Self {
            mint,
            authority,
            signers,
            transfer_fee_basis_points,
            maximum_fee,
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
            return Err(ProgramError::InvalidArgument);
        }

        // Instruction accounts.

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            // mint account
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.mint.address()));

            // authority account (single or multisig)
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::new(
                    self.authority.address(),
                    false,
                    self.signers.is_empty(),
                ));

            // multisig signer accounts
            for (instruction_account, signer) in instruction_accounts
                .get_unchecked_mut(2..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // Instruction.

        let expected_accounts = 2 + self.signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: &[
                ExtensionDiscriminator::TransferFee as u8,
                Self::DISCRIMINATOR,
            ],
        };

        // Accounts.

        let mut accounts = [UNINIT_ACCOUNT_REF; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            // mint account
            accounts.get_unchecked_mut(0).write(self.mint);

            // authority account (single or multisig)
            accounts.get_unchecked_mut(1).write(self.authority);

            // multisig signer accounts
            for (account, signer) in accounts
                .get_unchecked_mut(2..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(*signer);
            }
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
