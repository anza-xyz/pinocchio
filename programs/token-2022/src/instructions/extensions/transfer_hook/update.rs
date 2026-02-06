use {
    crate::{
        instructions::{extensions::ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
        instructions::{extensions::ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
        write_bytes, UNINIT_BYTE,
    },
    core::{
        mem::MaybeUninit,
        slice::{self, from_raw_parts},
    },
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Update the transfer hook program id. Only supported for mints that
/// include the `TransferHook` extension.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The mint.
///   1. `[signer]` The transfer hook authority.
///
///   * Multisignature authority
///   0. `[writable]` The mint.
///   1. `[]` The mint's transfer hook authority.
///   2. `..2+M` `[signer]` M signer accounts.
pub struct UpdateTransferHook<'a, 'b, 'c> {
    /// The token mint.
    pub mint: &'a AccountView,

    /// The mint's `withdraw_withheld_authority` or multisig.
    pub authority: &'a AccountView,

    /// Multisignature owner/delegate.
    pub signers: &'c [&'a AccountView],

    /// Program that authorizes the transfer.
    pub transfer_hook_program_id: Option<&'b Address>,

    /// The token program.
    pub token_program_id: &'b Address,
}

impl<'a, 'b, 'c> UpdateTransferHook<'a, 'b, 'c> {
    pub const DISCRIMINATOR: u8 = 1;

    /// Creates a new `UpdateTransferHook` instruction
    /// with a single owner/delegate authority.
    #[inline(always)]
    pub fn new(
        token_program_id: &'b Address,
        mint: &'a AccountView,
        authority: &'a AccountView,
        transfer_hook_program_id: Option<&'b Address>,
    ) -> Self {
        Self {
            mint,
            authority,
            signers: &[],
            transfer_hook_program_id,
            token_program_id,
        }
    }

    /// Creates a new `UpdateTransferHook` instruction with a
    /// multisignature owner/delegate authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig(
        token_program_id: &'b Address,
        mint: &'a AccountView,
        authority: &'a AccountView,
        transfer_hook_program_id: Option<&'b Address>,
        signers: &'c [&'a AccountView],
    ) -> Self {
        Self {
            mint,
            authority,
            signers,
            transfer_hook_program_id,
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
            Err(ProgramError::InvalidArgument)?;
        }

        let expected_accounts = 2 + self.signers.len();

        // Instruction accounts.

        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.mint.address()));

            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::new(
                    self.authority.address(),
                    false,
                    self.signers.is_empty(),
                ));

            for (account, signer) in instruction_accounts
                .get_unchecked_mut(2..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 34];

        // discriminators
        write_bytes(
            &mut instruction_data[..2],
            &[
                ExtensionDiscriminator::TransferHook as u8,
                Self::DISCRIMINATOR,
            ],
        );

        // transfer_hook_program_id
        write_bytes(
            &mut instruction_data[2..34],
            if let Some(program_id) = self.transfer_hook_program_id {
                program_id.as_ref()
            } else {
                &[0; 32]
            },
        );

        //Instruction.

        let instruction = InstructionView {
            program_id: self.token_program_id,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        // Accounts.

        const UNINIT_INFO: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut accounts = [UNINIT_INFO; 2 + MAX_MULTISIG_SIGNERS];

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
                account.write(signer);
            }
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { slice::from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
