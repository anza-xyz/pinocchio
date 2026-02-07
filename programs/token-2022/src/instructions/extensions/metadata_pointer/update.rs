use {
    crate::{
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

/// Update the metadata pointer address. Only supported for mints that
/// include the `MetadataPointer` extension.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The mint.
///   1. `[signer]` The metadata pointer authority.
///
///   * Multisignature authority
///   0. `[writable]` The mint.
///   1. `[]` The mint's metadata pointer authority.
///   2. `..2+M` `[signer]` M signer accounts.
pub struct Update<'a, 'b, 'c> {
    /// The mint.
    pub mint: &'a AccountView,

    /// The metadata pointer authority.
    pub authority: &'a AccountView,

    /// The signer accounts when `authority` is a multisig.
    pub signers: &'c [&'a AccountView],

    /// The new account address that holds the metadata.
    pub metadata_address: Option<&'b Address>,

    /// The token program.
    pub token_program: &'b Address,
}

impl Update<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 1;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.signers.len() > MAX_MULTISIG_SIGNERS {
            Err(ProgramError::InvalidArgument)?;
        }

        // Instruction accounts.

        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The expected number of accounts has been validated to be less than
        // the maximum allocated.
        unsafe {
            // mint
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.mint.address()));

            // authority
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
                ExtensionDiscriminator::MetadataPointer as u8,
                Update::DISCRIMINATOR,
            ],
        );
        // metadata_address
        write_bytes(
            &mut instruction_data[2..34],
            if let Some(address) = self.metadata_address {
                address.as_ref()
            } else {
                &[0; 32]
            },
        );

        let expected_accounts = 2 + self.signers.len();

        // Instruction.

        let instruction = InstructionView {
            program_id: self.token_program,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
            accounts: unsafe {
                slice::from_raw_parts(
                    instruction_accounts.as_ptr() as *const InstructionAccount,
                    expected_accounts,
                )
            },
        };

        // Accounts.

        const UNINIT_ACCOUNT_VIEWS: MaybeUninit<&AccountView> =
            MaybeUninit::<&AccountView>::uninit();
        let mut accounts = [UNINIT_ACCOUNT_VIEWS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The expected number of accounts has been validated to be less than
        // the maximum allocated.
        unsafe {
            // mint
            accounts.get_unchecked_mut(0).write(self.mint);

            // authority
            accounts.get_unchecked_mut(1).write(self.authority);

            // signer accounts
            for (account_view, signer) in accounts
                .get_unchecked_mut(2..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account_view.write(signer);
            }
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe {
                slice::from_raw_parts(accounts.as_ptr() as *const &AccountView, expected_accounts)
            },
            signers,
        )
    }
}
